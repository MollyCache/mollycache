use crate::cli::{ast::{interpreter::Interpreter, InsertIntoStatement, SqlStatement::{self, InsertInto}}, table::Value, tokenizer::{scanner::Token, token::TokenTypes}};
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
                                let blob_str = &token.value[2..token.value.len()-1]; // Remove X' and '
                                match decode(blob_str) {
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
    todo!()
}

fn or_statement(_interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    todo!()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tokenizer::scanner::Token;

    fn token(tt: TokenTypes, val: &'static str, col_num: usize) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: col_num,
            line_num: 1,
        }
    }

    #[test]
    fn simple_insert_works() {
        let tokens = vec![
            token(TokenTypes::Insert, "INSERT", 1),
            token(TokenTypes::Into, "INTO", 8),
            token(TokenTypes::Identifier, "users", 13),
            token(TokenTypes::Values, "VALUES", 19),
            token(TokenTypes::LeftParen, "(", 26),
            token(TokenTypes::IntLiteral, "1", 27),
            token(TokenTypes::Comma, ",", 28),
            token(TokenTypes::String, "\"Alice\"", 30),
            token(TokenTypes::RightParen, ")", 37),
            token(TokenTypes::SemiColon, ";", 38),
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
}