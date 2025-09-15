use crate::db::table::Value;
use crate::interpreter::{
    ast::{
        InsertIntoStatement,
        SqlStatement::{self, InsertInto},
        helpers::common::get_table_name,
        helpers::token::{expect_token_type, token_to_value},
        parser::Parser,
    },
    tokenizer::token::TokenTypes,
};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let statement: Result<SqlStatement, String>;
    let token = parser.current_token()?;
    match token.token_type {
        TokenTypes::Into => {
            statement = into_statement(parser);
        }
        TokenTypes::Or => {
            statement = or_statement(parser);
        }
        _ => return Err(parser.format_error()),
    }

    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;

    return statement;
}

fn into_statement(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let table_name = get_table_name(parser)?;

    let token = parser.current_token()?;
    let columns = match token.token_type {
        TokenTypes::LeftParen => Some(get_columns(parser)?),
        TokenTypes::Values => None,
        _ => return Err(parser.format_error()),
    };

    let mut values = vec![];

    let token = parser.current_token()?;
    if token.token_type == TokenTypes::Values {
        parser.advance()?;
        loop {
            values.push(get_values(parser)?);
            let token = parser.current_token()?;
            match token.token_type {
                TokenTypes::Comma => {
                    parser.advance()?;
                }
                _ => break,
            }
        }
    }

    let statement = InsertIntoStatement {
        table_name: table_name,
        columns: columns,
        values: values,
    };
    validate_insert_statement(&statement)?;
    return Ok(InsertInto(statement));
}

fn validate_insert_statement(statement: &InsertIntoStatement) -> Result<(), String> {
    for row in &statement.values {
        if row.len() != statement.values[0].len() {
            return Err(format!("Rows have different lengths"));
        }
    }

    if let Some(columns) = &statement.columns {
        if columns.len() != statement.values[0].len() {
            return Err(format!("Columns and values have different lengths"));
        }
    }
    return Ok(());
}

fn get_values(parser: &mut Parser) -> Result<Vec<Value>, String> {
    // Check for LeftParen
    expect_token_type(parser, TokenTypes::LeftParen)?;
    parser.advance()?;
    let mut values: Vec<Value> = vec![];
    loop {
        values.push(token_to_value(parser)?);
        parser.advance()?;

        let token = parser.current_token()?;
        match token.token_type {
            TokenTypes::Comma => {
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                parser.advance()?;
                return Ok(values);
            }
            _ => return Err(parser.format_error()),
        }
    }
}

fn get_columns(parser: &mut Parser) -> Result<Vec<String>, String> {
    parser.advance()?;
    let mut columns: Vec<String> = vec![];
    loop {
        let token = parser.current_token()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        columns.push(token.value.to_string());
        parser.advance()?;

        let token = parser.current_token()?;
        match token.token_type {
            TokenTypes::Comma => {
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                parser.advance()?;
                break;
            }
            _ => return Err(parser.format_error()),
        }
    }
    return Ok(columns);
}

fn or_statement(_parser: &mut Parser) -> Result<SqlStatement, String> {
    return Err("INSERT OR ... not yet implemented".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn single_row_insert_statement_is_generated_correctly() {
        // INSERT INTO users VALUES (1, "Alice");
        let tokens = vec![
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
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(
            statement,
            SqlStatement::InsertInto(InsertIntoStatement {
                table_name: "users".to_string(),
                columns: None,
                values: vec![vec![Value::Integer(1), Value::Text("Alice".to_string()),]],
            })
        );
    }

    #[test]
    fn multi_row_insert_statement_is_generated_correctly() {
        // INSERT INTO users VALUES (1, "Alice"), (2, "Bob");
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "2"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Bob"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(
            statement,
            SqlStatement::InsertInto(InsertIntoStatement {
                table_name: "guests".to_string(),
                columns: None,
                values: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string()),],
                    vec![Value::Integer(2), Value::Text("Bob".to_string()),]
                ],
            })
        );
    }

    #[test]
    fn single_row_insert_with_column_specifiers_is_generated_correctly() {
        // INSERT INTO users (id, name, email) VALUES (1, "Fletcher", NULL);
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "email"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::RealLiteral, "1.1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Blob, "AAB000"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Null, "NULL"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
            ]),
            values: vec![vec![
                Value::Real(1.1),
                Value::Blob(vec![0xAA, 0xB0, 0x00]),
                Value::Null,
            ]],
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn insert_into_without_table_name_is_error() {
        // INSERT INTO VALUES (1, "Alice");
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn insert_or_is_not_implemented_error() {
        // INSERT OR users VALUES (1, "Alice");
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn insert_with_different_lengths_is_error() {
        // INSERT INTO users VALUES (1, "Alice"), (2, "Bob", "Charlie");
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "2"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Bob"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Charlie"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
        let expected = Err("Rows have different lengths".to_string());
        assert_eq!(expected, result);
    }

    #[test]
    fn insert_with_different_column_and_value_lengths_is_error() {
        // INSERT INTO users (id, name) VALUES (1, "Alice", "Bob");
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Bob"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
        let expected = Err("Columns and values have different lengths".to_string());
        assert_eq!(expected, result);
    }
}
