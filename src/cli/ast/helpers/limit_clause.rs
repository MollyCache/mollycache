use crate::cli::ast::{parser::Parser, LimitClause};
use crate::cli::tokenizer::token::TokenTypes;
use crate::db::table::Value;
use crate::cli::ast::common::expect_token_type;
use crate::cli::ast::common::token_to_value;

pub fn get_limit(parser: &mut Parser) -> Result<Option<LimitClause>, String> { 
    if expect_token_type(parser, TokenTypes::Limit).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    expect_token_type(parser, TokenTypes::IntLiteral)?;
    let limit = token_to_value(parser)?;
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
    let offset = token_to_value(parser)?;
    if let Value::Integer(offset) = offset {
        if offset < 0 {
            return Err(parser.format_error());
        }
    };
    parser.advance()?;

    return Ok(Some(LimitClause {
        limit: limit,
        offset: Some(offset),
    }));
}