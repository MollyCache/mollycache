use crate::cli::{ast::{parser::Parser, common::token_to_value, InsertIntoStatement, SqlStatement::{self, InsertInto}}, table::Value, tokenizer::token::TokenTypes};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance();
    let statement: Result<SqlStatement, String>;
    match parser.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Into => {
                    statement = into_statement(parser);
                },
                TokenTypes::Or => {
                    statement = or_statement(parser);
                },
                _ => return Err(parser.format_error()),
            }
        },
        None => return Err(parser.format_error()),
    }
    // Ensure SemiColon
    match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::SemiColon {
                return Err(parser.format_error());
            }
        },
        None => return Err(parser.format_error()),
    }
    parser.advance(); // Move past the semicolon
    return statement;
}

fn into_statement(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance();
    let table_name = match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(parser.format_error());
            }
            let name = token.value.to_string();
            parser.advance();
            name
        },
        None => return Err(parser.format_error()),
    };
    let columns = match parser.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::LeftParen {
                Some(get_columns(parser)?)
            }
            else {
                None
            }
        },
        None => return Err(parser.format_error()),
    };
    let mut values = vec![];
    match parser.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::Values {
                parser.advance();
                loop {
                    values.push(get_values(parser)?);
                    match parser.current_token() {
                        Some(token) if token.token_type == TokenTypes::Comma => {
                            parser.advance();
                        },
                        Some(token) if token.token_type == TokenTypes::SemiColon => break,
                        _ => break,
                    }
                }
            }
            else {
                return Err(parser.format_error());
            }
        },
        None => return Err(parser.format_error()),
    };
    return Ok(InsertInto(InsertIntoStatement {
        table_name: table_name,
        columns: columns,
        values: values,
    }));
}

fn get_values(parser: &mut Parser) -> Result<Vec<Value>, String> {
    // Check for LeftParen
    match parser.current_token() {
        Some(token) if token.token_type == TokenTypes::LeftParen => {
            parser.advance();
            let mut values: Vec<Value> = vec![];
            loop {
                match parser.current_token() {
                    Some(_) => {
                        values.push(token_to_value(parser)?);
                        parser.advance();
                        match parser.current_token() {
                            Some(token) if token.token_type == TokenTypes::Comma => {
                                parser.advance();
                            },
                            Some(token) if token.token_type == TokenTypes::RightParen => {
                                parser.advance();
                                return Ok(values);
                            },
                            _ => return Err(parser.format_error()),
                        }
                    },
                    None => return Err(parser.format_error()),
                }
            }
        }
        _ => return Err(parser.format_error()),
    }
}

fn get_columns(parser: &mut Parser) -> Result<Vec<String>, String> {
    parser.advance();
    let mut columns: Vec<String> = vec![];
    loop {
        match parser.current_token() {
            Some(token) => {
                if token.token_type != TokenTypes::Identifier{
                    return Err(parser.format_error());
                }
                columns.push(token.value.to_string());
            }
            None => {
                return Err(parser.format_error())
            }
        }
        parser.advance();
        
        match parser.current_token() {
            Some(token) => {
                match token.token_type {
                    TokenTypes::Comma => {
                        parser.advance();
                    }
                    TokenTypes::RightParen => {
                        parser.advance();
                        break;
                    }
                    _ => {
                        return Err(parser.format_error())
                    }
                }
            }
            None => {
                return Err(parser.format_error());
            }
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
        assert_eq!(statement, SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![
                vec![
                    Value::Integer(1),
                    Value::Text("Alice".to_string()),
                ]
            ],
        }));
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
        assert_eq!(statement, SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "guests".to_string(),
            columns: None,
            values: vec![
                vec![
                    Value::Integer(1),
                    Value::Text("Alice".to_string()),
                ],
                vec![
                    Value::Integer(2),
                    Value::Text("Bob".to_string()),
                ]
            ],
        }));
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
        assert_eq!(statement, SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
            ]),
            values: vec![
                vec![
                    Value::Real(1.1),
                    Value::Blob(vec![0xAA, 0xB0, 0x00]),
                    Value::Null,
                ]
            ],
        }));       
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
}