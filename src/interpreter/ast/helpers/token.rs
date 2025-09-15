use crate::db::table::{DataType, Value};
use crate::interpreter::ast::helpers::common::hex_decode;
use crate::interpreter::ast::parser::Parser;
use crate::interpreter::tokenizer::scanner::Token;
use crate::interpreter::tokenizer::token::TokenTypes;

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
            let num = token
                .value
                .parse::<i64>()
                .map_err(|_| parser.format_error())?;
            Ok(Value::Integer(num))
        }
        TokenTypes::RealLiteral => {
            let num = token
                .value
                .parse::<f64>()
                .map_err(|_| parser.format_error())?;
            Ok(Value::Real(num))
        }
        TokenTypes::String => Ok(Value::Text(token.value.to_string())),
        TokenTypes::Blob => {
            let bytes = hex_decode(token.value).map_err(|_| parser.format_error())?;
            Ok(Value::Blob(bytes))
        }
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

pub fn token_to_data_type(parser: &mut Parser) -> Result<DataType, String> {
    let token = parser.current_token()?;
    return match token.token_type {
        TokenTypes::Integer => Ok(DataType::Integer),
        TokenTypes::Real => Ok(DataType::Real),
        TokenTypes::Text => Ok(DataType::Text),
        TokenTypes::Blob => Ok(DataType::Blob),
        TokenTypes::Null => Ok(DataType::Null),
        _ => Err(parser.format_error()),
    };
}

pub fn token_to_string(token: &Token) -> String {
    match token.token_type {
        TokenTypes::String => format!("'{}'", token.value),
        TokenTypes::HexLiteral => format!("X'{}'", token.value),
        TokenTypes::EOF
        | TokenTypes::SemiColon
        | TokenTypes::LeftParen
        | TokenTypes::RightParen => token.value.to_string(),
        _ => token.value.to_string() + " ",
    }
}

// TODO: Improve this function and the related code. Parsing tokens back into a string is a messy.
// This should be guarenteed to only be hit if the statement is valid.
pub fn format_statement_tokens(tokens: &[Token]) -> String {
    let mut result = String::new();
    for token in tokens {
        result += &token_to_string(token);
    }
    result = result.replace(" ;", ";").replace(" ,", ",");
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::parser::Parser;
    use crate::interpreter::ast::test_utils::token;
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
    fn format_statement_tokens_handles_single_token() {
        let tokens = vec![token(TokenTypes::SemiColon, ";")];
        let result = format_statement_tokens(&tokens);
        assert_eq!(";".to_string(), result);
    }

    #[test]
    fn format_statement_tokens_handles_multiple_tokens() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let result = format_statement_tokens(&tokens);
        assert_eq!("SELECT * FROM users;".to_string(), result);
    }
}
