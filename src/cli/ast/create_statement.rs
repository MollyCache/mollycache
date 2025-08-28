use crate::cli::{ast::{interpreter::Interpreter, CreateTableStatement, SqlStatement::{self, CreateTable}}, table::{ColumnDefinition, DataType}, tokenizer::token::TokenTypes};

pub fn build(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    let statement: Result<SqlStatement, String>;
    match interpreter.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Table => {
                    statement = table_statement(interpreter);
                },
                TokenTypes::Index => {
                    statement = index_statement(interpreter);
                },
                _ => return Err(interpreter.format_error()),
            }
        },
        None => return Err(interpreter.format_error()),
    }
    // Ensure SemiColon
    interpreter.advance();
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
                            columns.push(ColumnDefinition {
                                name: column_name,
                                data_type: column_data_type,
                                constraints: vec![] // TODO,
                            });
                        }
                        TokenTypes::RightParen => {
                            columns.push(ColumnDefinition {
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

fn index_statement(_interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    return Err("Index statements not yet implemented".to_string());
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
    fn create_table_generates_proper_statement(){
        let tokens = vec![
            token(TokenTypes::Create, "CREATE", 0),
            token(TokenTypes::Table, "TABLE", 7),
            token(TokenTypes::Identifier, "users", 13),
            token(TokenTypes::LeftParen, "(", 18),
            token(TokenTypes::Identifier, "id", 19),
            token(TokenTypes::Integer, "INTEGER", 22),
            token(TokenTypes::Comma, ",", 29),
            token(TokenTypes::Identifier, "name", 31),
            token(TokenTypes::Text, "TEXT", 36),
            token(TokenTypes::RightParen, ")", 40),
            token(TokenTypes::SemiColon, ";", 41),
            token(TokenTypes::EOF, "", 0),
        ];
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let tokens = vec![
            token(TokenTypes::Create, "CREATE", 0),
            token(TokenTypes::Table, "TABLE", 7),
            token(TokenTypes::Identifier, "users", 13),
            token(TokenTypes::LeftParen, "(", 18),
            token(TokenTypes::Identifier, "num", 19),
            token(TokenTypes::Integer, "REAL", 22),
            token(TokenTypes::Comma, ",", 29),
            token(TokenTypes::Identifier, "my_blob", 31),
            token(TokenTypes::Blob, "BLOB", 36),
            token(TokenTypes::Comma, ",", 29),
            token(TokenTypes::Identifier, "my_null", 31),
            token(TokenTypes::Null, "Null", 36),
            token(TokenTypes::RightParen, ")", 40),
            // Missing SemiColon
            token(TokenTypes::EOF, "", 0),
        ];
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        assert!(result.is_err());
    }

    #[test]
    fn create_table_with_bad_data_type() {
        let tokens = vec![
            token(TokenTypes::Create, "CREATE", 0),
            token(TokenTypes::Table, "TABLE", 7),
            token(TokenTypes::Identifier, "users", 13),
            token(TokenTypes::LeftParen, "(", 18),
            token(TokenTypes::Identifier, "id", 19),
            token(TokenTypes::Asterisk, "*", 22), // Bad Data Type
            token(TokenTypes::Comma, ",", 23),
            token(TokenTypes::Identifier, "name", 25),
            token(TokenTypes::Text, "TEXT", 30),
            token(TokenTypes::RightParen, ")", 34),
            token(TokenTypes::SemiColon, ";", 35),
            token(TokenTypes::EOF, "", 0),
        ];
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        assert!(result.is_err());
    }

    #[test]
    fn create_table_missing_comma() {
        let tokens = vec![
            token(TokenTypes::Create, "CREATE", 0),
            token(TokenTypes::Table, "TABLE", 7),
            token(TokenTypes::Identifier, "users", 13),
            token(TokenTypes::LeftParen, "(", 18),
            token(TokenTypes::Identifier, "id", 19),
            token(TokenTypes::Integer, "INTEGER", 22), // Missing Comma
            token(TokenTypes::Identifier, "name", 31),
            token(TokenTypes::Text, "TEXT", 36),
            token(TokenTypes::RightParen, ")", 40),
            token(TokenTypes::SemiColon, ";", 41),
            token(TokenTypes::EOF, "", 0),
        ];
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        assert!(result.is_err());
    }

    #[test]
    fn index_statement_not_implemented() {
        let tokens = vec![
            token(TokenTypes::Create, "CREATE", 0),
            token(TokenTypes::Index, "INDEX", 7),
            token(TokenTypes::Identifier, "my_index", 13),
            token(TokenTypes::SemiColon, ";", 22),
            token(TokenTypes::EOF, "", 0),
        ];
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        assert!(result.is_err());
    }
}