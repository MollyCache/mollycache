use crate::cli::ast::{
    helpers::common::{expect_token_type, token_to_value},
    parser::Parser,
    Operator, WhereCondition, WhereStackElement, LogicalOperator, Parentheses, WhereStackOperators,
};
use crate::cli::tokenizer::token::TokenTypes;

// The WhereStack is a the method that is used to store the order of operations with Reverse Polish Notation.
// This is built from the infix expression of the where clause. Using the shunting yard algorithm. Thanks Djikstra!
// Operator precedence is given as '()' > 'NOT' > 'AND' > 'OR'
// This is currently represented as stack of LogicalOperators and WhereConditions.
// WhereConditions are currently represented as 'column operator value'
// This will later be expanded to replace the WhereConditions with a generalized evaluation function.

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<Vec<WhereStackElement>>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;
    let mut where_stack: Vec<WhereStackElement> = vec![];

    let mut operator_stack: Vec<WhereStackOperators> = vec![];

    loop {
        let where_condition = get_where_condition(parser)?;
        match where_condition {
            Some(where_stack_element) => {
                match where_stack_element {
                    WhereStackElement::Condition(where_condition) => {
                        where_stack.push(WhereStackElement::Condition(where_condition));
                    },
                    WhereStackElement::Parentheses(parentheses) => {
                        match parentheses {
                            Parentheses::Left => {
                                operator_stack.push(WhereStackOperators::Parentheses(parentheses));
                            },
                            Parentheses::Right => {
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
                                _ => return Err("Mismatched parentheses found.".to_string()),
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
        assert!(result.is_ok());
        let where_clause = result.unwrap();
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
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }
}
