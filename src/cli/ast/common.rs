use crate::cli::{ast::parser::Parser, table::Value, tokenizer::token::TokenTypes};
use hex::decode;

pub fn token_to_value(parser: &Parser) -> Result<Value, String> {
    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    
    match token.token_type {
        TokenTypes::IntLiteral => {
            let num = token.value.parse::<i64>()
                .map_err(|_| parser.format_error())?;
            Ok(Value::Integer(num))
        },
        TokenTypes::RealLiteral => {
            let num = token.value.parse::<f64>()
                .map_err(|_| parser.format_error())?;
            Ok(Value::Real(num))
        },
        TokenTypes::String => Ok(Value::Text(token.value.to_string())),
        TokenTypes::Blob => {
            let bytes = decode(token.value)
                .map_err(|_| parser.format_error())?;
            Ok(Value::Blob(bytes))
        },
        TokenTypes::Null => Ok(Value::Null),
        _ => Err(parser.format_error()),
    }
}

// Returns a list of Strings from the tokens when they are formated as "identifier, identifier, ..."
pub fn tokens_to_identifier_list(parser: &mut Parser) -> Result<Vec<String>, String> {
    let mut identifiers: Vec<String> = vec![];
    loop {
        let token = parser.current_token().ok_or_else(|| parser.format_error())?;
        if token.token_type != TokenTypes::Identifier {
            return Err(parser.format_error());
        }
        identifiers.push(token.value.to_string());
        parser.advance();
        let token = parser.current_token().ok_or_else(|| parser.format_error())?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        parser.advance();
    }
    return Ok(identifiers);
}