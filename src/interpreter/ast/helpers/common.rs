use crate::interpreter::{
    ast::{ExistenceCheck, parser::Parser},
    tokenizer::token::TokenTypes,
};

// Re-export get_selectables from its dedicated module
pub use super::selectables::get_selectables::get_selectables;

// Returns an error if the current token does not match the given token type
pub fn expect_token_type(parser: &Parser, token_type: TokenTypes) -> Result<(), String> {
    let token = parser.current_token()?;
    if token.token_type != token_type {
        return Err(parser.format_error());
    }
    Ok(())
}

// Returns Ok(actual_table_name, alias defaulted to "") or an error
pub fn get_table_name(parser: &mut Parser) -> Result<(String, String), String> {
    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let mut result = (token.value.to_string(), "".to_string());
    parser.advance()?;

    if let Ok(next_token) = parser.current_token()
        && next_token.token_type == TokenTypes::As
    {
        parser.advance()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        result.1 = parser.current_token()?.value.to_string();
        parser.advance()?;
    }

    Ok(result)
}

pub fn exists_clause(
    parser: &mut Parser,
    check_type: ExistenceCheck,
) -> Result<Option<ExistenceCheck>, String> {
    if parser.current_token()?.token_type == TokenTypes::If {
        parser.advance()?;
        let token = parser.current_token()?;
        let existence_check = match (&token.token_type, check_type) {
            (TokenTypes::Not, ExistenceCheck::IfNotExists) => {
                parser.advance()?;
                expect_token_type(parser, TokenTypes::Exists)?;
                ExistenceCheck::IfNotExists
            }
            (TokenTypes::Exists, ExistenceCheck::IfExists) => ExistenceCheck::IfExists,
            (_, _) => return Err(parser.format_error()),
        };
        parser.advance()?;
        return Ok(Some(existence_check));
    }
    return Ok(None);
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at {}: {}", i, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_handles_valid_hex_string() {
        let result = hex_decode("0A1A3F");
        assert!(result.is_ok());
        let expected = vec![0x0A, 0x1A, 0x3F];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn decode_handles_invalid_hex_string() {
        let result = hex_decode("0AZA3A");
        assert!(result.is_err());
        let expected = "Invalid hex at 2: invalid digit found in string";
        assert_eq!(expected, result.err().unwrap());

        let result = hex_decode("0A1");
        assert!(result.is_err());
        let expected = "Hex string must have even length";
        assert_eq!(expected, result.err().unwrap());
    }

    #[test]
    fn exists_clause_handles_all_cases() {
        use crate::interpreter::ast::test_utils::token;
        use crate::interpreter::ast::{ExistenceCheck, parser::Parser};

        let tokens = vec![
            token(TokenTypes::If, "IF"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Exists, "EXISTS"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = exists_clause(&mut parser, ExistenceCheck::IfNotExists);
        assert!(result.is_ok());
        assert_eq!(Some(ExistenceCheck::IfNotExists), result.unwrap());

        let tokens = vec![
            token(TokenTypes::If, "IF"),
            token(TokenTypes::Exists, "EXISTS"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = exists_clause(&mut parser, ExistenceCheck::IfExists);
        assert!(result.is_ok());
        assert_eq!(Some(ExistenceCheck::IfExists), result.unwrap());

        let tokens = vec![
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = exists_clause(&mut parser, ExistenceCheck::IfNotExists);
        assert!(result.is_ok());
        assert_eq!(None, result.unwrap());

        let tokens = vec![
            token(TokenTypes::If, "IF"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::EOF, ""),
        ];
        let mut parser = Parser::new(tokens);
        let result = exists_clause(&mut parser, ExistenceCheck::IfNotExists);
        assert!(result.is_err());
    }

    #[test]
    fn get_table_name_handles_aliases() {
        use crate::interpreter::ast::parser::Parser;
        use crate::interpreter::ast::test_utils::token;

        let tokens = vec![
            token(TokenTypes::Identifier, "some_table_name"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "some_alias"),
        ];
        let mut parser = Parser::new(tokens);
        let result_with_alias = get_table_name(&mut parser);
        assert_eq!(
            result_with_alias,
            Ok(("some_table_name".to_string(), "some_alias".to_string()))
        );
    }

    #[test]
    fn get_table_name_handles_no_aliases() {
        use crate::interpreter::ast::parser::Parser;
        use crate::interpreter::ast::test_utils::token;

        let tokens = vec![token(TokenTypes::Identifier, "some_table_name")];
        let mut parser = Parser::new(tokens);
        let result_with_alias = get_table_name(&mut parser);
        assert_eq!(
            result_with_alias,
            Ok(("some_table_name".to_string(), "".to_string()))
        );
    }
}
