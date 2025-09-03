use crate::cli::{ast::{parser::Parser, WhereClause, Operator, OrderByClause, OrderByDirection, LimitClause}, tokenizer::token::TokenTypes};

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


pub fn get_order_by(parser: &mut Parser) -> Result<Option<Vec<OrderByClause>>, String> {
    if expect_token_type(parser, TokenTypes::Order).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    expect_token_type(parser, TokenTypes::By)?;
    parser.advance()?;

    let mut order_by_clauses = vec![];
    loop {
        let token = parser.current_token()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        let column = token.value.to_string();
        parser.advance()?;

        let token = parser.current_token()?;
        let direction = match token.token_type {
            TokenTypes::Asc => {
                parser.advance()?;
                OrderByDirection::Asc
            },
            TokenTypes::Desc => {
                parser.advance()?;
                OrderByDirection::Desc
            },
            _ => OrderByDirection::Asc,
        };

        order_by_clauses.push(OrderByClause {
            column: column,
            direction: direction,
        });

        let token = parser.current_token()?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        parser.advance()?;
    }
    return Ok(Some(order_by_clauses));
}

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