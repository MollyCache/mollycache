use crate::cli::{ast::{interpreter::Interpreter, CreateTableStatement, SqlStatement, SqlStatement::CreateTable}, tokenizer::token::TokenTypes};

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
    todo!()
}

fn index_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    todo!()
}