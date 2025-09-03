use crate::cli::ast::{parser::Parser, WhereClause, Operator, common::{expect_token_type, token_to_value}};
use crate::cli::tokenizer::token::TokenTypes;

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<WhereClause>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let column = token.value.to_string();
    parser.advance()?;

    let token = parser.current_token()?;
    let operator  = match token.token_type {
        TokenTypes::Equals => Operator::Equals,
        TokenTypes::NotEquals => Operator::NotEquals,
        TokenTypes::LessThan => Operator::LessThan,
        TokenTypes::LessEquals => Operator::LessEquals,
        TokenTypes::GreaterThan => Operator::GreaterThan,
        TokenTypes::GreaterEquals => Operator::GreaterEquals,
        _ => return Err(parser.format_error()),
    };
    parser.advance()?;

    let value = token_to_value(parser)?;
    parser.advance()?;

    return Ok(Some(WhereClause {
        column: column,
        operator: operator,
        value: value,
    }));
}