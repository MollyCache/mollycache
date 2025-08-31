use crate::cli::tokenizer::{scanner::Token, token::TokenTypes};
use crate::db::table::{ColumnDefinition, Value};

mod common;
mod create_statement;
mod insert_statement;
mod parser;
mod select_statement;

#[derive(Debug, PartialEq)]
pub enum SqlStatement {
    CreateTable(CreateTableStatement),
    InsertInto(InsertIntoStatement),
    Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq)]
pub struct InsertIntoStatement {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Value>>,
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub table_name: String,
    pub columns: SelectStatementColumns,
    pub where_clause: Option<WhereClause>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub enum SelectStatementColumns {
    All,
    Specific(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEquals,
    GreaterEquals,
}

#[derive(Debug, PartialEq)]
pub struct WhereClause {
    pub column: String,
    pub operator: Operator,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq)]
pub struct OrderByClause {
    pub column: String,
    pub direction: OrderByDirection,
}

#[derive(Debug, PartialEq)]
pub struct LimitClause {
    pub limit: Value,
    pub offset: Option<Value>,
}

pub trait StatementBuilder {
    fn build_create(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_insert(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_select(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
}

pub struct DefaultStatementBuilder;

impl StatementBuilder for DefaultStatementBuilder {
    fn build_create(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        create_statement::build(parser)
    }
    
    fn build_insert(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        insert_statement::build(parser)
    }
    
    fn build_select(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        select_statement::build(parser)
    }
}

pub fn generate(tokens: Vec<Token>) -> Vec<Result<SqlStatement, String>> {
    let mut results: Vec<Result<SqlStatement, String>> = vec![];
    let mut parser = parser::Parser::new(tokens);
    let builder : &dyn StatementBuilder = &DefaultStatementBuilder;
    loop {
        let next_statement = parser.next_statement(builder);
        if let Some(next_statement) = next_statement {
            if next_statement.is_err() {
                loop {
                    if let Ok(token) = parser.current_token() {
                        if token.token_type != TokenTypes::EOF && token.token_type != TokenTypes::SemiColon {
                           let _ = parser.advance();
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        break;
                    }
                }
            }
            let parser_advance_result = parser.advance_past_semicolon();
            if parser_advance_result.is_err() {
                results.push(Err(parser_advance_result.err().unwrap()));
                return results;
            }
            results.push(next_statement);
        } else {
            break;
        }
    }
    return results;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tokenizer::scanner::Token;

    fn token(tt: TokenTypes, val: &'static str) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: 0,
            line_num: 1,
        }
    }

    #[test]
    fn ast_handles_invalid_statements_gracefully() {
        let tokens = vec![
            token(TokenTypes::Select, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_err());
        let expected = vec![Err("Error at line 1, column 0: Unexpected token type: Into".to_string())];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_multiple_statements() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_ok());
        assert!(result[1].is_ok());
        let expected = vec![
            Ok(SqlStatement::Select(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            })),
            Ok(SqlStatement::InsertInto(InsertIntoStatement {
                table_name: "users".to_string(),
                columns: None,
                values: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                ],
            })),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_invalid_statement_then_valid_statement() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        println!("{:?}", result);
        assert!(result[0].is_err());
        assert!(result[1].is_ok());
        let expected = vec![
            Err("Error at line 1, column 0: Unexpected token type: SemiColon".to_string()),
            Ok(SqlStatement::InsertInto(InsertIntoStatement {
        
                table_name: "users".to_string(),
                columns: None,
                values: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                ],
            })),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_multiple_valid_statements() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_ok());
        assert!(result[1].is_ok());
        let expected = vec![
            Ok(SqlStatement::Select(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            })),
            Ok(SqlStatement::InsertInto(InsertIntoStatement {
                table_name: "users".to_string(),
                columns: None,
                values: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                ],
            })),
        ];
        assert_eq!(expected, result);
    }
}