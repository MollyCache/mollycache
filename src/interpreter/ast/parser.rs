use crate::interpreter::{
    ast::{
        SqlStatement,
        helpers::token::format_statement_tokens,
        statement_builder::{DefaultStatementBuilder, StatementBuilder},
    },
    tokenizer::scanner::Token,
    tokenizer::token::TokenTypes,
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    builder: &'a dyn StatementBuilder,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        return Self {
            tokens,
            start: 0,
            current: 0,
            builder: &DefaultStatementBuilder {},
        };
    }

    pub fn line_num(&self) -> Result<usize, String> {
        return Ok(self.current_token()?.line_num);
    }

    pub fn current_token(&self) -> Result<&Token<'a>, String> {
        if self.current >= self.tokens.len() {
            return Err(self.format_error());
        }
        return Ok(&self.tokens[self.current]);
    }

    pub fn peek_token(&self) -> Result<&Token<'a>, String> {
        if self.current + 1 >= self.tokens.len() {
            return Err(self.format_error());
        }
        return Ok(&self.tokens[self.current + 1]);
    }

    pub fn get_sql_statement_text(&self) -> String {
        return format_statement_tokens(&self.tokens[self.start..self.current]);
    }

    pub fn advance(&mut self) -> Result<(), String> {
        if let Ok(token) = self.current_token() {
            if token.token_type == TokenTypes::SemiColon {
                return Err(self.format_error());
            }
        }
        self.current += 1;
        Ok(())
    }

    pub fn advance_past_semicolon(&mut self) -> Result<(), String> {
        if let Ok(token) = self.current_token() {
            if token.token_type == TokenTypes::SemiColon {
                self.current += 1;
                return Ok(());
            }
        }
        return Err("Expected token type: SemiColon was not found".to_string());
    }

    pub fn format_error(&self) -> String {
        if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            return format!(
                "Error at line {:?}, column {:?}: Unexpected value: {}",
                token.line_num,
                token.col_num,
                token.value.to_string()
            );
        } else {
            return "Error at end of input.".to_string();
        }
    }

    pub fn format_error_nearby(&self) -> String {
        if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            return format!(
                "Error near line {:?}, column {:?}",
                token.line_num, token.col_num
            );
        } else {
            return "Error at end of input.".to_string();
        }
    }

    pub fn next_statement(&mut self) -> Option<Result<SqlStatement, String>> {
        self.start = self.current;
        match (&self.current_token(), &self.peek_token()) {
            (Ok(token), Ok(peek_token)) => match (&token.token_type, &peek_token.token_type) {
                (TokenTypes::Create, _) => Some(self.builder.build_create(self)),
                (TokenTypes::Insert, _) => Some(self.builder.build_insert(self)),
                (TokenTypes::Select, _) | (TokenTypes::LeftParen, TokenTypes::Select) => {
                    Some(self.builder.build_select(self))
                }
                (TokenTypes::Update, _) => Some(self.builder.build_update(self)),
                (TokenTypes::Delete, _) => Some(self.builder.build_delete(self)),
                (TokenTypes::Drop, _) => Some(self.builder.build_drop(self)),
                (TokenTypes::Alter, _) => Some(self.builder.build_alter(self)),
                (TokenTypes::Begin, _) => Some(self.builder.build_begin(self)),
                (TokenTypes::Commit, _) | (TokenTypes::End, _) => {
                    Some(self.builder.build_commit(self))
                }
                (TokenTypes::Rollback, _) => Some(self.builder.build_rollback(self)),
                (TokenTypes::Savepoint, _) => Some(self.builder.build_savepoint(self)),
                (TokenTypes::Release, _) => Some(self.builder.build_release(self)),
                _ => Some(Err(self.format_error())),
            },
            (Ok(token), Err(_)) => match token.token_type {
                TokenTypes::EOF => None,
                _ => Some(Err(self.format_error_nearby())),
            },
            _ => Some(Err(self.format_error())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::statement_builder::MockStatementBuilder;
    use crate::interpreter::ast::test_utils::{token, token_with_location};
    use crate::interpreter::ast::{
        CreateTableStatement, InsertIntoStatement, SelectMode, SelectStatement,
        SelectStatementColumn, SelectStatementStack, SelectStatementStackElement, SelectableStack,
        SelectableStackElement, SelectStatementTable,
    };

    #[test]
    fn parser_formats_error_when_at_end_of_input() {
        let tokens = vec![];
        let parser = Parser::new(tokens);
        let result = parser.format_error();
        assert_eq!(result, "Error at end of input.");
    }

    #[test]
    fn parser_formats_error_when_unexpected_token_type() {
        let tokens = vec![token_with_location(TokenTypes::Insert, "INSERT", 15, 3)];
        let parser = Parser::new(tokens);
        let result = parser.format_error();
        assert_eq!(
            result,
            "Error at line 3, column 15: Unexpected value: INSERT"
        );
    }

    #[test]
    fn parser_next_statement_filters_options_correctly_handles_multiple_statements() {
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser {
            tokens,
            start: 0,
            current: 0,
            builder: &MockStatementBuilder,
        };
        // Create Table
        let result = parser.next_statement();
        let expected = Some(Ok(SqlStatement::CreateTable(CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
            columns: vec![],
        })));
        assert_eq!(result, expected);

        // Insert Into
        let result = parser.next_statement();
        let expected = Some(Ok(SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![],
        })));
        assert_eq!(result, expected);

        // Select
        let result = parser.next_statement();
        let expected = Some(Ok(SqlStatement::Select(SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(
                SelectStatement {
                    table_name: SelectStatementTable::new("users".to_string()),
                    mode: SelectMode::All,
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All],
                    },
                    column_names: vec![SelectStatementColumn::new("*".to_string())],
                    where_clause: None,
                    order_by_clause: None,
                    limit_clause: None,
                },
            )],
            order_by_clause: None,
            limit_clause: None,
        })));
        assert_eq!(result, expected);

        // EOF
        let result = parser.next_statement();
        let expected = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn parser_next_statement_handles_errors_correctly() {
        let tokens = vec![
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser {
            tokens,
            start: 0,
            current: 0,
            builder: &MockStatementBuilder,
        };
        let result = parser.next_statement();
        let expected = Some(Err(
            "Error at line 1, column 0: Unexpected value: users".to_string()
        ));
        assert_eq!(result, expected);
    }
}
