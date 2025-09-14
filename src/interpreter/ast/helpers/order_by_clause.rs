use crate::interpreter::ast::helpers::common::get_selectables;
use crate::interpreter::ast::{parser::Parser, OrderByClause, OrderByDirection, SelectableStack, SelectableStackElement};
use crate::interpreter::tokenizer::token::TokenTypes;

use crate::interpreter::ast::helpers::token::expect_token_type;

pub fn get_order_by(parser: &mut Parser) -> Result<Option<OrderByClause>, String> {
    if expect_token_type(parser, TokenTypes::Order).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    expect_token_type(parser, TokenTypes::By)?;
    parser.advance()?;

    let mut directions = vec![];
    let mut column_names = vec![];
    let columns = get_selectables(parser, true, &mut Some(&mut directions), &mut Some(&mut column_names))?;
    
    return Ok(Some(OrderByClause {
        columns: columns,
        column_names: column_names,
        directions: directions
    }));
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
        let expected = Some(OrderByClause {
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())]
            },
            column_names: vec!["id".to_string()],
            directions: vec![OrderByDirection::Asc],
        });
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
        let expected = Some(OrderByClause {
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Column("name".to_string())
                ]
            },
            column_names: vec!["id".to_string(), "name".to_string()],
            directions: vec![OrderByDirection::Asc, OrderByDirection::Desc],
        });
        assert_eq!(expected, order_by_clause);
    }
}