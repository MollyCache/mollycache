use crate::cli::{ast::{interpreter::{self, Interpreter}, InsertIntoStatement, SqlStatement::{self, InsertInto}}, table::Value, tokenizer::token::TokenTypes};
use hex::decode;

pub fn build(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    let statement: Result<SqlStatement, String>;
    match interpreter.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Into => {
                    statement = into_statement(interpreter);
                },
                TokenTypes::Or => {
                    statement = or_statement(interpreter);
                },
                _ => return Err(interpreter.format_error()),
            }
        },
        None => return Err(interpreter.format_error()),
    }
    // Ensure SemiColon
    match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::SemiColon {
                return Err(interpreter.format_error());
            }
        },
        None => return Err(interpreter.format_error()),
    }

    return statement;
}

fn into_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    let table_name = match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(interpreter.format_error());
            }
            let name = token.value.to_string();
            interpreter.advance();
            name
        },
        None => return Err(interpreter.format_error()),
    };
    let columns = match interpreter.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::LeftParen {
                Some(get_columns(interpreter)?)
            }
            else {
                None
            }
        },
        None => return Err(interpreter.format_error()),
    };
    let mut values = vec![];
    match interpreter.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::Values {
                interpreter.advance();
                loop {
                    values.push(get_values(interpreter)?);
                    match interpreter.current_token() {
                        Some(token) if token.token_type == TokenTypes::Comma => {
                            interpreter.advance();
                        },
                        Some(token) if token.token_type == TokenTypes::SemiColon => break,
                        _ => break,
                    }
                }
            }
            else {
                return Err(interpreter.format_error());
            }
        },
        None => return Err(interpreter.format_error()),
    };
    return Ok(InsertInto(InsertIntoStatement {
        table_name: table_name,
        columns: columns,
        values: values,
    }));
}

fn get_values(interpreter: &mut Interpreter) -> Result<Vec<Value>, String> {
    // Check for LeftParen
    match interpreter.current_token() {
        Some(token) if token.token_type == TokenTypes::LeftParen => {
            interpreter.advance();
            let mut values: Vec<Value> = vec![];
            loop {
                match interpreter.current_token() {
                    Some(token) => {
                        match token.token_type { 
                            TokenTypes::IntLiteral => {
                                match token.value.parse::<i64>() {
                                    Ok(num) => values.push(Value::Integer(num)),
                                    Err(_) => return Err(interpreter.format_error()),
                                }
                                interpreter.advance();
                            },
                            TokenTypes::RealLiteral => {
                                match token.value.parse::<f64>() {
                                    Ok(num) => values.push(Value::Real(num)),
                                    Err(_) => return Err(interpreter.format_error()),
                                }
                                interpreter.advance();
                            },
                            TokenTypes::String => {
                                values.push(Value::Text(token.value.to_string()));
                                interpreter.advance();
                            },
                            TokenTypes::Blob => {
                                match decode(token.value) {
                                    Ok(bytes) => values.push(Value::Blob(bytes)),
                                    Err(_) => return Err(interpreter.format_error()),
                                }
                                interpreter.advance();
                            },
                            TokenTypes::Null => {
                                values.push(Value::Null);
                                interpreter.advance();
                            },
                            _ => return Err(interpreter.format_error()),
                        }
                        match interpreter.current_token() {
                            Some(token) if token.token_type == TokenTypes::Comma => {
                                interpreter.advance();
                            },
                            Some(token) if token.token_type == TokenTypes::RightParen => {
                                interpreter.advance();
                                return Ok(values);
                            },
                            _ => return Err(interpreter.format_error()),
                        }
                    },
                    None => return Err(interpreter.format_error()),
                }
            }
        }
        _ => return Err(interpreter.format_error()),
    }
}

fn get_columns(interpreter: &mut Interpreter) -> Result<Vec<String>, String> {
    interpreter.advance();
    let mut columns: Vec<String> = vec![];
    loop {
        match interpreter.current_token() {
            Some(token) => {
                if token.token_type != TokenTypes::Identifier{
                    return Err(interpreter.format_error());
                }
                columns.push(token.value.to_string());
            }
            None => {
                return Err(interpreter.format_error())
            }
        }
        interpreter.advance();
        
        match interpreter.current_token() {
            Some(token) => {
                match token.token_type {
                    TokenTypes::Comma => {
                        interpreter.advance();
                    }
                    TokenTypes::RightParen => {
                        interpreter.advance();
                        break;
                    }
                    _ => {
                        return Err(interpreter.format_error())
                    }
                }
            }
            None => {
                return Err(interpreter.format_error());
            }
        }
    }
    return Ok(columns);
}

fn or_statement(_interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    todo!()
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
}