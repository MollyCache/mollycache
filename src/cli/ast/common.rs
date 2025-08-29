use crate::cli::{ast::interpreter::Interpreter, table::Value, tokenizer::token::TokenTypes};
use hex::decode;

pub fn token_to_value(interpreter: &Interpreter) -> Result<Value, String> {
    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    
    match token.token_type {
        TokenTypes::IntLiteral => {
            let num = token.value.parse::<i64>()
                .map_err(|_| interpreter.format_error())?;
            Ok(Value::Integer(num))
        },
        TokenTypes::RealLiteral => {
            let num = token.value.parse::<f64>()
                .map_err(|_| interpreter.format_error())?;
            Ok(Value::Real(num))
        },
        TokenTypes::String => Ok(Value::Text(token.value.to_string())),
        TokenTypes::Blob => {
            let bytes = decode(token.value)
                .map_err(|_| interpreter.format_error())?;
            Ok(Value::Blob(bytes))
        },
        TokenTypes::Null => Ok(Value::Null),
        _ => Err(interpreter.format_error()),
    }
}

// Returns a list of Strings from the tokens when they are formated as "identifier, identifier, ..."
pub fn tokens_to_identifier_list(interpreter: &mut Interpreter) -> Result<Vec<String>, String> {
    let mut identifiers: Vec<String> = vec![];
    loop {
        let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
        if token.token_type != TokenTypes::Identifier {
            return Err(interpreter.format_error());
        }
        identifiers.push(token.value.to_string());
        interpreter.advance();
        let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        interpreter.advance();
    }
    return Ok(identifiers);
}