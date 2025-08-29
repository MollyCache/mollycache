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
    match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::SemiColon {
                return Err(interpreter.format_error());
            }
        },
        None => return Err(interpreter.format_error()),
    }
    interpreter.advance(); // Move past the semicolon
    return statement;
}

fn table_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    let table_name = match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(interpreter.format_error());
            }
            token.value.to_string()
            
        },
        None => return Err(interpreter.format_error()),
    };
    interpreter.advance();

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
                            interpreter.advance();
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        assert!(result.is_err());
    }
}