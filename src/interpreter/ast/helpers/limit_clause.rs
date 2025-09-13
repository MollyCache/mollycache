use crate::interpreter::ast::{parser::Parser, LimitClause};
use crate::interpreter::tokenizer::token::TokenTypes;
use crate::db::table::Value;
use crate::interpreter::ast::helpers::token::{expect_token_type, token_to_value};

pub fn get_limit(parser: &mut Parser) -> Result<Option<LimitClause>, String> { 
    if expect_token_type(parser, TokenTypes::Limit).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    expect_token_type(parser, TokenTypes::IntLiteral)?;
    let limit = match token_to_value(parser)? {
        Value::Integer(v) => {
            if v < 0 {
                return Err("Invalid LIMIT value: must be positive.".to_string())
            }
            v as usize
        },
        _ => return Err("Invalid LIMIT value: must be integer.".to_string())
    };
    parser.advance()?;

    let token = parser.current_token()?;
    if token.token_type != TokenTypes::Offset {
        return Ok(Some(LimitClause {
            limit: limit,
            offset: None,
        }));
    }
    parser.advance()?;

    expect_token_type(parser, TokenTypes::IntLiteral)?;
    let offset = match token_to_value(parser)? {
        Value::Integer(v) => {
            if v < 0 {
                return Err("Invalid OFFSET value: must be positive.".to_string())
            }
            v as usize
        },
        _ => return Err("Invalid OFFSET value: must be integer.".to_string())
    };
    parser.advance()?;

    return Ok(Some(LimitClause {
        limit: limit,
        offset: Some(offset),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn limit_clause_is_generated_correctly() {
        // LIMIT 10 OFFSET 5;
        let tokens = vec![
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_limit(&mut parser);
        assert!(result.is_ok());
        let limit_clause = result.unwrap();
        let expected = Some(LimitClause {
            limit: 10,
            offset: Some(5),
        });
        assert_eq!(expected, limit_clause);
    }

    #[test]
    fn limit_clause_with_no_offset_is_generated_correctly() {
        // LIMIT 10;
        let tokens = vec![
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_limit(&mut parser);
        assert!(result.is_ok());
        let limit_clause = result.unwrap();
        let expected = Some(LimitClause {
            limit: 10,
            offset: None,
        });
        assert_eq!(expected, limit_clause);
    }

    #[test]
    fn not_limit_clause_returns_none() {
        // SELECT * ...;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_limit(&mut parser);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Select);
    }

    #[test]
    fn limit_clause_with_negative_offset_is_generated_correctly() {
        // LIMIT 10 OFFSET -5;
        let tokens = vec![
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "-5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_limit(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error at line 1, column 0: Unexpected value: -5");
    }
}