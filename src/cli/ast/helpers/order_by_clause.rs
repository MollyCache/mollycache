use crate::cli::ast::{parser::Parser, OrderByClause, OrderByDirection};
use crate::cli::tokenizer::token::TokenTypes;

use crate::cli::ast::common::expect_token_type;

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