use crate::cli::{ast::{parser::Parser, CreateTableStatement, SqlStatement::{self, CreateTable}, common::expect_token_type}, tokenizer::token::TokenTypes};
use crate::db::table::{ColumnDefinition, DataType};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let statement: Result<SqlStatement, String>;
    
    let token = parser.current_token()?;
    match token.token_type {
        TokenTypes::Table => {
            statement = table_statement(parser);
        },
        TokenTypes::Index => {
            statement = index_statement(parser);
        },
        _ => return Err(parser.format_error()),
    }

    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return statement;
}

fn table_statement(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;

    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let table_name = token.value.to_string();
    parser.advance()?;

    let column_definitions = column_definitions(parser)?;
    return Ok(CreateTable(CreateTableStatement {
        table_name,
        columns: column_definitions,
    }));
}

fn column_definitions(parser: &mut Parser) -> Result<Vec<ColumnDefinition>, String> {
    let mut columns: Vec<ColumnDefinition> = vec![];

    expect_token_type(parser, TokenTypes::LeftParen)?;
    parser.advance()?;

    loop {
        let token = parser.current_token()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        let column_name = token.value.to_string();
        parser.advance()?;
        
        // Grab the column data type
        let column_data_type = token_to_data_type(parser)?;
        parser.advance()?;

        // TODO: Modifiers and Constraints

        // Ensure we have a comma or right paren
        let token = parser.current_token()?;
        match token.token_type {
            TokenTypes::Comma => {
                columns.push(ColumnDefinition {
                    name: column_name,
                    data_type: column_data_type,
                    constraints: vec![] // TODO,
                });
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                columns.push(ColumnDefinition {
                    name: column_name,
                    data_type: column_data_type,
                    constraints: vec![] // TODO,
                });
                parser.advance()?;
                break;
            },
            _ => return Err(parser.format_error()),
        }
    }
    return Ok(columns);
}

fn token_to_data_type(parser: &mut Parser) -> Result<DataType, String> {
    let token = parser.current_token()?;
    return match token.token_type {
        TokenTypes::Integer => Ok(DataType::Integer),
        TokenTypes::Real => Ok(DataType::Real),
        TokenTypes::Text => Ok(DataType::Text),
        TokenTypes::Blob => Ok(DataType::Blob),
        TokenTypes::Null => Ok(DataType::Null),
        _ => Err(parser.format_error()),
    };
}

fn index_statement(_parser: &mut Parser) -> Result<SqlStatement, String> {
    return Err("Index statements not yet implemented".to_string());
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
    fn create_table_generates_proper_statement(){
        // CREATE TABLE users (id INTEGER, name TEXT);
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Integer, "INTEGER"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Text, "TEXT"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        let expected = SqlStatement::CreateTable(CreateTableStatement {
            table_name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![],
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Text,
                    constraints: vec![],
                },
            ],
        });
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn create_table_statement_missing_semicolon() {
        // CREATE TABLE users (num REAL, my_blob BLOB, my_null NULL)
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "num"),
            token(TokenTypes::Integer, "REAL"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "my_blob"),
            token(TokenTypes::Blob, "BLOB"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "my_null"),
            token(TokenTypes::Null, "Null"),
            token(TokenTypes::RightParen, ")"),
            // Missing SemiColon
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn create_table_with_bad_data_type() {
        // CREATE TABLE users (id *, name TEXT);
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asterisk, "*"), // Bad Data Type
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Text, "TEXT"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn create_table_missing_comma() {
        // CREATE TABLE users (id INTEGER name TEXT);
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Integer, "INTEGER"), // Missing Comma
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Text, "TEXT"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn index_statement_not_implemented() {
        // CREATE INDEX my_index;
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Index, "INDEX"),
            token(TokenTypes::Identifier, "my_index"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }
}