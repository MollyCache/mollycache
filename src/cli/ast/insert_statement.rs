use crate::cli::{ast::{interpreter::{self, Interpreter}, SqlStatement}, tokenizer::token::TokenTypes};

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

fn into_statement(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();

}

fn or_statement(_interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    todo!()
}