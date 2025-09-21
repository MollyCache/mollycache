use crate::db::table::core::column::ColumnDefinition;
use crate::interpreter::{
    ast::{
        CreateTableStatement, ExistenceCheck,
        SqlStatement::{self, CreateTable},
        helpers::common::{exists_clause, get_table_name},
        helpers::token::{expect_token_type, token_to_data_type},
        parser::Parser,
    },
    tokenizer::token::TokenTypes,
};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let statement: Result<SqlStatement, String>;

    let token = parser.current_token()?;
    match token.token_type {
        TokenTypes::Table => {
            statement = table_statement(parser);
        }
        _ => return Err(parser.format_error()),
    }

    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return statement;
}

fn table_statement(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let existence_check = exists_clause(parser, ExistenceCheck::IfNotExists)?;

    let table_name = get_table_name(parser, false)?.table_name;

    let column_definitions = column_definitions(parser)?;
    return Ok(CreateTable(CreateTableStatement {
        table_name,
        existence_check,
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
                    constraints: vec![], // TODO,
                });
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                columns.push(ColumnDefinition {
                    name: column_name,
                    data_type: column_data_type,
                    constraints: vec![], // TODO,
                });
                parser.advance()?;
                break;
            }
            _ => return Err(parser.format_error()),
        }
    }
    return Ok(columns);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::DataType;
    use crate::interpreter::ast::ExistenceCheck;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn create_table_generates_proper_statement() {
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
            existence_check: None,
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
    fn create_table_with_if_exists_clause() {
        // CREATE TABLE IF NOT EXISTS users (id INTEGER);
        let tokens = vec![
            token(TokenTypes::Create, "CREATE"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::If, "IF"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Exists, "EXISTS"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Integer, "INTEGER"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        let expected = SqlStatement::CreateTable(CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: Some(ExistenceCheck::IfNotExists),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: DataType::Integer,
                constraints: vec![],
            }],
        });
        assert_eq!(expected, result.unwrap());
    }
}
