use crate::cli::{
    ast::{SqlStatement, StatementBuilder},
    tokenizer::scanner::Token, tokenizer::token::TokenTypes,
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        return Self { 
            tokens, 
            current: 0, 
        };
    }
    
    pub fn current_token(&self) -> Result<&Token<'a>, String> {
        if self.current >= self.tokens.len() {
            return Err(self.format_error());
        }
        return Ok(&self.tokens[self.current]);
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
                token.line_num, token.col_num, token.value.to_string()
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

    pub fn next_statement(&mut self, builder: &dyn StatementBuilder) -> Option<Result<SqlStatement, String>> {
        match self.current_token() {
            Ok(token) => match token.token_type {
                TokenTypes::Create => Some(builder.build_create(self)),
                TokenTypes::Insert => Some(builder.build_insert(self)),
                TokenTypes::Select => Some(builder.build_select(self)),
                TokenTypes::Update => Some(builder.build_update(self)),
                TokenTypes::Delete => Some(builder.build_delete(self)),
                TokenTypes::EOF => None,
                _ => {
                    Some(Err(self.format_error()))
                }
            },
            Err(error) => Some(Err(error)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::{CreateTableStatement, InsertIntoStatement, SelectStatement, SelectStatementColumns};
    use crate::cli::ast::test_utils::{token_with_location, token};

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
        assert_eq!(result, "Error at line 3, column 15: Unexpected value: INSERT");
    }

    pub struct MockStatementBuilder;

    impl StatementBuilder for MockStatementBuilder {
        fn build_create(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
            parser.advance()?;
            parser.advance_past_semicolon()?;
            return Ok(SqlStatement::CreateTable(CreateTableStatement {
                table_name: "users".to_string(),
                columns: vec![],
            }));
        }
        
        fn build_insert(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
            parser.advance()?;
            parser.advance_past_semicolon()?;
            return Ok(SqlStatement::InsertInto(InsertIntoStatement {
                table_name: "users".to_string(),
                columns: None,
                values: vec![],
            }));
        }
        
        fn build_select(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
            parser.advance()?;
            parser.advance_past_semicolon()?;
            return Ok(SqlStatement::Select(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }));
        }

        fn build_update(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
            todo!();
        }

        fn build_delete(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
            todo!();
        }
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
        let mut parser = Parser::new(tokens);
        let builder : &dyn StatementBuilder = &MockStatementBuilder;
        // Create Table
        let result = parser.next_statement(builder);
        let expected = Some(Ok(SqlStatement::CreateTable(CreateTableStatement {
            table_name: "users".to_string(),
            columns: vec![],
        })));
        assert_eq!(result, expected);

        // Insert Into
        let result = parser.next_statement(builder);
        let expected = Some(Ok(SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![],
        })));
        assert_eq!(result, expected);

        // Select
        let result = parser.next_statement(builder);
        let expected = Some(Ok(SqlStatement::Select(SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        })));
        assert_eq!(result, expected);

        // EOF
        let result = parser.next_statement(builder);
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
        let mut parser = Parser::new(tokens);
        let builder : &dyn StatementBuilder = &MockStatementBuilder;
        let result = parser.next_statement(builder);
        let expected = Some(Err("Error at line 1, column 0: Unexpected value: users".to_string()));
        assert_eq!(result, expected);
    }
}
