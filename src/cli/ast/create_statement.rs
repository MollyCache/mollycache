use std::num::Saturating;

use crate::cli::{ast::{interpreter::Interpreter, CreateTableStatement, SqlStatement::{self, CreateTable}}, table::{ColumnDefinition, DataType}, tokenizer::{scanner::Token, token::TokenTypes}};

pub fn build(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    match interpreter.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Table => {
                    return table_statement(interpreter);
                },
                TokenTypes::Index => {
                    return index_statement(interpreter);
                },
                _ => return Err(interpreter.format_error()),
            }
        },
        None => return Err(interpreter.format_error()),
    }
}

fn table_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
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
    let column_definitions = column_definitions(interpreter)?;
    return Ok(CreateTable(CreateTableStatement {
        table_name,
        columns: column_definitions,
    }));
}

fn column_definitions(interpreter: &mut Interpreter) -> Result<Vec<ColumnDefinition>, String> {
    let mut columns: Vec<crate::cli::table::ColumnDefinition> = vec![];
    if let Some(token) = interpreter.current_token() {
        if token.token_type != TokenTypes::LeftParen {
            return Err(interpreter.format_error());
        }
        else {
            interpreter.advance();
            loop {
                let column_name = match interpreter.current_token() {
                    Some(token) => {
                        if token.token_type != TokenTypes::Identifier {
                            return Err(interpreter.format_error());
                        }
                        token.value.to_string()
                    },
                    None => return Err(interpreter.format_error()),
                };
                interpreter.advance();
                
                // Grab the column data type
                let column_data_type = token_to_data_type(interpreter)?;
                interpreter.advance();

                // TODO: Modifiers and Constraints

                // Ensure we have a comma or right paren
                if let Some(token) = interpreter.current_token() {
                    match &token.token_type {
                        TokenTypes::Comma => {
                            columns.push(crate::cli::table::ColumnDefinition {
                                name: column_name,
                                data_type: column_data_type,
                                constraints: vec![] // TODO,
                            });
                        }
                        TokenTypes::RightParen => {
                            columns.push(crate::cli::table::ColumnDefinition {
                                name: column_name,
                                data_type: column_data_type,
                                constraints: vec![] // TODO,
                            });
                            break;
                        },
                        _ => return Err(interpreter.format_error()),
                    }
                } else {
                    return Err(interpreter.format_error());
                }
                interpreter.advance();
            }
            return Ok(columns);
        }
    } else {
        return Err(interpreter.format_error());
    }
}

fn token_to_data_type(interpreter: &mut Interpreter) -> Result<DataType, String> {
    if let Some(token) = interpreter.current_token() {
        match token.token_type {
            TokenTypes::Integer => {
                return Ok(DataType::Integer);
            },
            TokenTypes::Real => {
                return Ok(DataType::Real);
            },
            TokenTypes::Text => {
                return Ok(DataType::Text);
            },
            TokenTypes::Blob => {
                return Ok(DataType::Blob);
            },
            TokenTypes::Null => {
                return Ok(DataType::Null);
            },
            _ => {
                return Err(interpreter.format_error());
            }
        }
    } else {
        return Err(interpreter.format_error());
    }
}

fn index_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    todo!()
}