use crate::interpreter::ast::{parser::Parser, OrderByClause, OrderByDirection};
use crate::interpreter::tokenizer::token::TokenTypes;

use crate::interpreter::ast::helpers::common::expect_token_type;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn order_by_clause_is_generated_correctly() {
        // ORDER BY id ASC LIMIT...;
        let tokens = vec![
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asc, "ASC"),
            token(TokenTypes::Limit, "LIMIT"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_order_by(&mut parser);
        assert!(result.is_ok());
        let order_by_clause = result.unwrap();
        let expected = Some(vec![OrderByClause {
            column: "id".to_string(),
            direction: OrderByDirection::Asc,
        }]);
        assert_eq!(expected, order_by_clause);
    }

    #[test]
    fn not_order_by_clause_returns_none() {
        // SELECT * ...;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_order_by(&mut parser);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Select);
    }

    #[test]
    fn order_by_clause_with_multiple_columns_is_generated_correctly() {
        // ORDER BY id ASC, name DESC;
        let tokens = vec![
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asc, "ASC"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Desc, "DESC"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_order_by(&mut parser);
        assert!(result.is_ok());
        let order_by_clause = result.unwrap();
        let expected = Some(vec![OrderByClause {
            column: "id".to_string(),
            direction: OrderByDirection::Asc,
        }, OrderByClause {
            column: "name".to_string(),
            direction: OrderByDirection::Desc,
        }]);
        assert_eq!(expected, order_by_clause);
    }
}