use crate::cli::ast::{parser::Parser, WhereTreeNode, WhereTreeEdge, WhereTreeElement, Operator, LogicalOperator, helpers::common::{expect_token_type, token_to_value}};
use crate::cli::tokenizer::token::TokenTypes;

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<WhereTreeNode>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    let left_where_tree_edge = Some(WhereTreeElement::Edge(get_where_clause_edge(parser)?));
    let operator = match parser.current_token()?.token_type {
        TokenTypes::And => Some(LogicalOperator::And),
        TokenTypes::Or => Some(LogicalOperator::Or),
        _ => None,
    };
    let right_where_tree_edge = match operator.is_some() {
        true => {
            parser.advance()?;
            Some(WhereTreeElement::Edge(get_where_clause_edge(parser)?))
        },
        false => None,
    };

    return Ok(Some(WhereTreeNode {
        left: Box::new(left_where_tree_edge),
        right: Box::new(right_where_tree_edge),
        operator: operator,
        negation: false,
    }));
}

fn get_where_clause_edge(parser: &mut Parser) -> Result<WhereTreeEdge, String> {
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

    return Ok(WhereTreeEdge {
        column: column,
        operator: operator,
        value: value,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tokenizer::scanner::Token;
    use crate::db::table::Value;

    fn token(tt: TokenTypes, val: &'static str) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: 0,
            line_num: 1,
        }
    }

    #[test]
    fn where_clause_with_all_tokens_is_generated_correctly() {
        // WHERE id = 1 LIMIT...
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Limit, "LIMIT"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        let expected = Some(WhereTreeNode {
            left: Box::new(Some(WhereTreeElement::Edge(WhereTreeEdge {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }))),
            right: Box::new(None),
            operator: None,
            negation: false,
        });
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Limit);
    }

    #[test]
    fn not_where_clause_returns_none() {
        // SELECT * ...;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Select);
    }

    #[test]
    fn where_clause_with_two_conditions_is_generated_correctly() {
        // WHERE id = 1 AND name = "John";
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        let expected = Some(WhereTreeNode {
            left: Box::new(Some(WhereTreeElement::Edge(WhereTreeEdge {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }))),
            right: Box::new(Some(WhereTreeElement::Edge(WhereTreeEdge {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }))),
            operator: Some(LogicalOperator::And),
            negation: false,
        });
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }
}
