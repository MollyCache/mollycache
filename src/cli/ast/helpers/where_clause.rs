use crate::cli::ast::{
    helpers::common::{expect_token_type, token_to_value},
    parser::Parser,
    Operator, WhereCondition, WhereStackElement, LogicalOperator, Parentheses, WhereStackOperators,
};
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
        TokenTypes::Identifier => {
            let column = token.value.to_string();
            parser.advance()?;

            let token = parser.current_token()?;
            let operator = match token.token_type {
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

            return Ok(Some(WhereStackElement::Condition(
                WhereCondition {
                    column,
                    operator,
                    value,
                })
            ));
        }
        _ => return Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::LogicalOperator;
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
        let expected = Some(vec![WhereStackElement::Condition(WhereCondition {
            column: "id".to_string(),
            operator: Operator::Equals,
            value: Value::Integer(1),
        })]);
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
        let expected = Some(vec![
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn where_clause_with_not_logical_operators_is_generated_correctly() {
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
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::Condition(WhereCondition {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
            WhereStackElement::Condition(WhereCondition {
                column: "age".to_string(),
                operator: Operator::GreaterThan,
                value: Value::Integer(20),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ]);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn where_clause_with_different_precedence_is_generated_correctly() {
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
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::Condition(WhereCondition {
                column: "age".to_string(),
                operator: Operator::GreaterThan,
                value: Value::Integer(20),
            }),
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
    fn where_clause_with_parentheses_is_generated_correctly() {
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
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
            WhereStackElement::Condition(WhereCondition {
                column: "age".to_string(),
                operator: Operator::GreaterThan,
                value: Value::Integer(20),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "active".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(0),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
        ]);
        println!("{:?}", result);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn where_clause_with_nested_parentheses_and_logical_operators_is_generated_correctly() {
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
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            WhereStackElement::Condition(WhereCondition {
                column: "age".to_string(),
                operator: Operator::GreaterThan,
                value: Value::Integer(20),
            }),
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
    fn where_clause_with_invalid_parentheses_is_generated_correctly() {
        // WHERE (id = 1 OR name = "John";
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
    fn where_clause_with_invalid_right_paren_is_generated_correctly() {
        // WHERE (id = 1 OR name = "John"));
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
    fn where_clause_with_valid_not_logical_operator_is_generated_correctly() {
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
            WhereStackElement::Condition(WhereCondition {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
        ]));
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn where_clause_with_invalid_not_logical_operator_is_generated_correctly() {
        // WHERE NOT AND id = 1;
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
