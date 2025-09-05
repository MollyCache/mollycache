use crate::cli::{ast::{
    helpers::{common::expect_token_type, where_condition::get_condition}, 
    parser::Parser, LogicalOperator, WhereStackElement, WhereStackOperators, Parentheses}};
use crate::cli::tokenizer::token::TokenTypes;

// The WhereStack is a the method that is used to store the order of operations with Reverse Polish Notation.
// This is built from the infix expression of the where clause. Using the shunting yard algorithm. Thanks Djikstra!
// Operator precedence is given as '()' > 'NOT' > 'AND' > 'OR'
// This is currently represented as stack of LogicalOperators, WhereConditions.
// WhereConditions are currently represented as 'column operator value'
// This will later be expanded to replace the WhereConditions with a generalized evaluation function.

// We also validate the order using an enum of the current next expected token types.
#[derive(PartialEq, Debug)]
enum WhereClauseExpectedNextToken {
    ConditionLeftParenNot,
    LogicalOperatorRightParen,
}

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<Vec<WhereStackElement>>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;
    let mut where_stack: Vec<WhereStackElement> = vec![];
    let mut operator_stack: Vec<WhereStackOperators> = vec![];
    let mut expected_next_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;

    loop {
        let where_condition = get_where_condition(parser)?;
        match where_condition {
            Some(where_stack_element) => {
                match where_stack_element {
                    WhereStackElement::Condition(where_condition) => {
                        if expected_next_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                            return Err(parser.format_error_nearby());
                        }
                        expected_next_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
                        where_stack.push(WhereStackElement::Condition(where_condition));
                    },
                    WhereStackElement::Parentheses(parentheses) => {
                        match parentheses {
                            Parentheses::Left => {
                                if expected_next_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                                    return Err(parser.format_error_nearby());
                                }
                                operator_stack.push(WhereStackOperators::Parentheses(parentheses));
                            },
                            Parentheses::Right => {
                                if expected_next_token != WhereClauseExpectedNextToken::LogicalOperatorRightParen {
                                    return Err(parser.format_error_nearby());
                                }
                                expected_next_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
                                loop {
                                    let current_operator = operator_stack.pop();
                                    if let Some (current_operator) = current_operator {
                                        match current_operator {
                                            WhereStackOperators::Parentheses(Parentheses::Left) => {
                                                break;
                                            },
                                            WhereStackOperators::LogicalOperator(logical_operator) => {
                                               where_stack.push(WhereStackElement::LogicalOperator(logical_operator));
                                            },
                                            WhereStackOperators::Parentheses(Parentheses::Right) => {
                                                return Err("Mismatched parentheses found.".to_string());
                                            },
                                        }
                                    }
                                    else {
                                        return Err("Mismatched parentheses found.".to_string());
                                    }
                                }   
                            },
                        }
                    },
                    WhereStackElement::LogicalOperator(logical_operator) => {
                        match logical_operator {
                            LogicalOperator::Not => {
                                if expected_next_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                                    return Err(parser.format_error_nearby());
                                }
                                expected_next_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
                            }
                            _ => {
                                if expected_next_token != WhereClauseExpectedNextToken::LogicalOperatorRightParen {
                                    return Err(parser.format_error_nearby());
                                }
                                expected_next_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
                            }
                        }
                        loop {
                            let current_operator =  if let Some(operator) = operator_stack.pop() {
                                operator
                            } else {
                                operator_stack.push(WhereStackOperators::LogicalOperator(logical_operator));
                                break;
                            };
                            match current_operator {
                                WhereStackOperators::LogicalOperator(current_operator) => {
                                    if logical_operator.is_greater_precedence(&current_operator) {
                                        operator_stack.push(WhereStackOperators::LogicalOperator(current_operator));
                                        operator_stack.push(WhereStackOperators::LogicalOperator(logical_operator));
                                        break;
                                    }
                                    else {
                                        where_stack.push(WhereStackElement::LogicalOperator(current_operator));
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
            None => break
        }
    }
    while let Some(operator) = operator_stack.pop() {
        match operator {
            WhereStackOperators::LogicalOperator(logical_operator) => {
                where_stack.push(WhereStackElement::LogicalOperator(logical_operator));
            },
            _ => return Err("Mismatched parentheses found.".to_string()),
        }
    }

    Ok(Some(where_stack))
}

fn get_where_condition(parser: &mut Parser) -> Result<Option<WhereStackElement>, String> {
    let token = parser.current_token()?;
    match token.token_type {
        // Logical operators and parentheses
        TokenTypes::And => {
            parser.advance()?;
            return Ok(Some(WhereStackElement::LogicalOperator(LogicalOperator::And)))
        },
        TokenTypes::Or => {
            parser.advance()?;
            return Ok(Some(WhereStackElement::LogicalOperator(LogicalOperator::Or)))
        },
        TokenTypes::Not => {
            parser.advance()?;
            return Ok(Some(WhereStackElement::LogicalOperator(LogicalOperator::Not)))
        },
        TokenTypes::LeftParen => {
            parser.advance()?;
            return Ok(Some(WhereStackElement::Parentheses(Parentheses::Left)))
        },
        TokenTypes::RightParen => {
            parser.advance()?;
            return Ok(Some(WhereStackElement::Parentheses(Parentheses::Right)))
        },
        // Conditions
        TokenTypes::Identifier | TokenTypes::IntLiteral | TokenTypes::RealLiteral | TokenTypes::String | TokenTypes::Blob | TokenTypes::Null => {
            return Ok(Some(WhereStackElement::Condition(get_condition(parser)?)));
        }
        _ => return Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::{LogicalOperator, Operator, WhereCondition, Operand};
    use crate::cli::ast::test_utils::token;
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
    fn returns_error_for_extra_closing_parenthesis() {
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
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Mismatched parentheses found.");
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
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
