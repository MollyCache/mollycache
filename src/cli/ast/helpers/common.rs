use crate::cli::{ast::{parser::Parser}, tokenizer::token::TokenTypes};

use crate::db::table::Value;
use hex::decode;

// Returns an error if the current token does not match the given token type
pub fn expect_token_type(parser: &Parser, token_type: TokenTypes) -> Result<(), String> {
    let token = parser.current_token()?;
    if token.token_type != token_type {
        return Err(parser.format_error());
    }
    Ok(())
}

pub fn token_to_value(parser: &Parser) -> Result<Value, String> {
    let token = parser.current_token()?;
    
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

// Returns a list of Values from the tokens when they are formated as "value, value, ..."
pub fn tokens_to_value_list(parser: &mut Parser) -> Result<Vec<Value>, String> {
    let mut values: Vec<Value> = vec![];
    loop {
        values.push(token_to_value(parser)?);
        parser.advance()?;
        let token = parser.current_token()?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        parser.advance()?;
    }
    return Ok(values);
}

// Returns a list of Strings from the tokens when they are formated as "identifier, identifier, ..."
pub fn tokens_to_identifier_list(parser: &mut Parser) -> Result<Vec<String>, String> {
    let mut identifiers: Vec<String> = vec![];
    loop {
        let token = parser.current_token()?;
        expect_token_type(parser, TokenTypes::Identifier)?;

        identifiers.push(token.value.to_string());
        parser.advance()?;
        let token = parser.current_token()?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        parser.advance()?;
    }
    return Ok(identifiers);
}

pub fn get_table_name(parser: &mut Parser) -> Result<String, String> {
    parser.advance()?;
    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let result = token.value.to_string();
    Ok(result)
}