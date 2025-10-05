use crate::interpreter::ast::helpers::common::get_selectables;
use crate::interpreter::ast::{OrderByClause, parser::Parser};
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
    let columns = get_selectables(parser, true, false, &mut Some(&mut directions))?;

    return Ok(Some(OrderByClause {
        columns: columns,
        directions: directions,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::{OrderByDirection, SelectableColumn, SelectableStackElement};

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
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
                column_name: "id".to_string(),
            }],
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
        assert_eq!(
            parser.current_token().unwrap().token_type,
            TokenTypes::Select
        );
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
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                },
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("name".to_string())],
                    column_name: "name".to_string(),
                },
            ],
            directions: vec![OrderByDirection::Asc, OrderByDirection::Desc],
        });
        assert_eq!(expected, order_by_clause);
    }
}
