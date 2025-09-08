use crate::interpreter::{ast::{parser::Parser, ExistenceCheck}, tokenizer::token::TokenTypes};

use crate::db::table::Value;

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
    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let result = token.value.to_string();
    parser.advance()?;
    Ok(result)
}

pub fn exists_clause(parser: &mut Parser, check_type: ExistenceCheck) -> Result<Option<ExistenceCheck>, String> {
    if parser.current_token()?.token_type == TokenTypes::If {
        parser.advance()?;
        let token = parser.current_token()?;
        let existence_check = match (&token.token_type, check_type) {
            (TokenTypes::Not, ExistenceCheck::IfNotExists) => {
                parser.advance()?;
                expect_token_type(parser, TokenTypes::Exists)?;
                ExistenceCheck::IfNotExists
            },
            (TokenTypes::Exists, ExistenceCheck::IfExists) => {
                ExistenceCheck::IfExists
            },
            (_, _) => return Err(parser.format_error()),
        };
        parser.advance()?;
        return Ok(Some(existence_check));
    }
    return Ok(None);
}

fn decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..hex.len()).step_by(2).map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at {}: {}", i, e))
        }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::parser::Parser;
    use crate::interpreter::tokenizer::token::TokenTypes;

    #[test]
    fn value_list_handles_single_value() {
        // 1);...
        let tokens = vec![
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::RightParen, ")"),
        ];
        let mut parser = Parser::new(tokens);
        let result = tokens_to_value_list(&mut parser);
        assert_eq!(result, Ok(vec![Value::Integer(1)]));
    }

    #[test]
    fn decode_handles_valid_hex_string() {
        let result = decode("0A1A3F");
        assert!(result.is_ok());
        let expected = vec![0x0A, 0x1A, 0x3F];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn decode_handles_invalid_hex_string() {
        let result = decode("0AZA3A");
        assert!(result.is_err());
        let expected = "Invalid hex at 2: invalid digit found in string";
        assert_eq!(expected, result.err().unwrap());
        
        let result = decode("0A1");
        assert!(result.is_err());
        let expected = "Hex string must have even length";
        assert_eq!(expected, result.err().unwrap());
    }
}