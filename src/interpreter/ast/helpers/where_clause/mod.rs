mod where_condition;
mod expected_token_matches_current;

use expected_token_matches_current::{next_expected_token_from_current, WhereClauseExpectedNextToken};

use crate::interpreter::{ast::{
    helpers::{common::expect_token_type, where_clause::where_condition::get_condition}, 
    parser::Parser, LogicalOperator, WhereStackElement, WhereStackOperators, Parentheses}};
use crate::interpreter::tokenizer::token::TokenTypes;

// The WhereStack is a the method that is used to store the order of operations with Reverse Polish Notation.
// This is built from the infix expression of the where clause. Using the shunting yard algorithm. Thanks Djikstra!
// Operator precedence is given as '()' > 'NOT' > 'AND' > 'OR'
// This is currently represented as stack of LogicalOperators, WhereConditions.
// WhereConditions are currently represented as 'column operator value'
// This will later be expanded to replace the WhereConditions with a generalized evaluation function.
pub fn get_where_clause(parser: &mut Parser) -> Result<Option<Vec<WhereStackElement>>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;
    let mut where_stack: Vec<WhereStackElement> = vec![];
    let mut operator_stack: Vec<WhereStackOperators> = vec![];
    let mut expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;

    while let Some(where_stack_element) = get_where_stack_element(parser, &operator_stack)? {
        expected_token = next_expected_token_from_current(&expected_token, &where_stack_element, parser)?;
        match where_stack_element {
            WhereStackElement::Condition(condition) => where_stack.push(WhereStackElement::Condition(condition)),
            WhereStackElement::Parentheses(parentheses) => {
                if parentheses == Parentheses::Left {
                    operator_stack.push(WhereStackOperators::Parentheses(parentheses));
                    continue;
                }
                while let Some(current_operator) = operator_stack.pop() {
                    match (current_operator, operator_stack.len()) {
                        (WhereStackOperators::LogicalOperator(_), 0) => return Err("Mismatched parentheses found.".to_string()),
                        (WhereStackOperators::Parentheses(Parentheses::Left), _) => break,
                        (WhereStackOperators::LogicalOperator(logical_operator), _) => where_stack.push(WhereStackElement::LogicalOperator(logical_operator)),
                        _ => unreachable!(),
                    }
                }
            },
            WhereStackElement::LogicalOperator(logical_operator) => {
                loop { 
                    let current_operator =  match operator_stack.pop() {
                        Some(operator) => operator,
                        None => {
                            operator_stack.push(WhereStackOperators::LogicalOperator(logical_operator));
                            break;
                        },
                    };
                    match current_operator {
                        WhereStackOperators::LogicalOperator(current_logical_operator) => {
                            if !logical_operator.is_greater_precedence(&current_logical_operator) {
                                where_stack.push(WhereStackElement::LogicalOperator(current_logical_operator));
                            }
                            else {
                                operator_stack.push(WhereStackOperators::LogicalOperator(current_logical_operator));
                                operator_stack.push(WhereStackOperators::LogicalOperator(logical_operator));
                                break;
                            }
                        },
                        _ => {
                            operator_stack.push(current_operator);
                            operator_stack.push(WhereStackOperators::LogicalOperator(logical_operator));
                            break;
                        },
                    }
                }
            }
        }
    }
    while let Some(operator) = operator_stack.pop() {
        match operator {
            WhereStackOperators::LogicalOperator(_) => where_stack.push(operator.into_where_stack_element()),
            _ => return Err("Mismatched parentheses found.".to_string()),
        }
    }

    Ok(Some(where_stack))
}

fn get_where_stack_element(parser: &mut Parser, operator_stack: &Vec<WhereStackOperators>) -> Result<Option<WhereStackElement>, String> {
    let token_type = &parser.current_token()?.token_type;    
    match token_type {
        TokenTypes::And | TokenTypes::Or | TokenTypes::Not | TokenTypes::LeftParen | TokenTypes::RightParen => {
            if token_type == &TokenTypes::RightParen && !operator_stack.contains(&WhereStackOperators::Parentheses(Parentheses::Left)) {
                return Ok(None); // Mismatched parens can be caused by the UNION STATEMENTs.
            }
            let where_stack_element = token_type_to_where_stack_element(token_type);
            parser.advance()?;
            Ok(Some(where_stack_element))
        },
        TokenTypes::Identifier | TokenTypes::IntLiteral | TokenTypes::RealLiteral | TokenTypes::String | TokenTypes::Blob | TokenTypes::Null => {
            return Ok(Some(WhereStackElement::Condition(get_condition(parser)?)));
        }
        _ => return Ok(None),
    }
}

fn token_type_to_where_stack_element(token_type: &TokenTypes) -> WhereStackElement {
    match token_type {
        TokenTypes::And => WhereStackElement::LogicalOperator(LogicalOperator::And),
        TokenTypes::Or => WhereStackElement::LogicalOperator(LogicalOperator::Or),
        TokenTypes::Not => WhereStackElement::LogicalOperator(LogicalOperator::Not),
        TokenTypes::LeftParen => WhereStackElement::Parentheses(Parentheses::Left),
        TokenTypes::RightParen => WhereStackElement::Parentheses(Parentheses::Right),
        _ => unreachable!("Invalid token type for where stack element"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{LogicalOperator, Operator, WhereCondition, Operand};
    use crate::interpreter::ast::test_utils::token;
    use crate::db::table::Value;

    fn simple_condition(l_side: &str, operator: Operator, r_side: Value) -> WhereStackElement {
        WhereStackElement::Condition(WhereCondition {
            l_side: Operand::Identifier(l_side.to_string()),
            operator: operator,
            r_side: Operand::Value(r_side),
        })
    }

    #[test]
    fn returns_none_when_no_where_keyword_present() {
        // SELECT * ... (no WHERE clause)
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
    fn parses_and_condition_with_correct_rpn_order() {
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
        let expected = Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn respects_logical_operator_precedence() {
        // WHERE NOT id = 1 AND name = "John" OR age > 20;
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "20"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        let expected = Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
            simple_condition("age", Operator::GreaterThan, Value::Integer(20)),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn handles_complex_operator_precedence_correctly() {
        // WHERE id = 1 OR NOT name = "John" AND NOT age > 20;
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "20"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        let expected = Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            simple_condition("age", Operator::GreaterThan, Value::Integer(20)),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn processes_parentheses_grouping_correctly() {
        // WHERE (id = 1 OR name = "John") AND NOT (age > 20 OR active = 0);
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "20"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "active"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "0"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        let expected = Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
            simple_condition("age", Operator::GreaterThan, Value::Integer(20)),
            simple_condition("active", Operator::Equals, Value::Integer(0)),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn handles_deeply_nested_parentheses() {
        // WHERE (id = 1 OR NOT (name = "John" AND age > 20));
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "20"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        let expected = Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            simple_condition("age", Operator::GreaterThan, Value::Integer(20)),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn returns_error_for_missing_closing_parenthesis() {
        // WHERE (id = 1 OR name = "John"; (missing closing parenthesis)
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Mismatched parentheses found.");
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn does_not_return_error_for_extra_closing_parenthesis() {
        // This extra closing parenthesis could be from the UNION STATEMENTs and would error on that level.
        // WHERE (id = 1 OR name = "John")); (extra closing parenthesis)
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ]));
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::RightParen);
    }

    #[test]
    fn parses_not_operator_in_simple_condition() {
        // WHERE NOT id = 1;
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(where_clause, Some(vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
        ]));
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn returns_error_for_invalid_not_operator_usage() {
        // WHERE NOT AND id = 1; (invalid: NOT followed by AND)
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error near line 1, column 0");
    }
}
