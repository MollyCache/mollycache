use crate::interpreter::ast::{WhereStackElement, LogicalOperator, Parentheses, parser::Parser};


#[derive(PartialEq, Debug)]
pub enum WhereClauseExpectedNextToken {
    ConditionLeftParenNot,
    LogicalOperatorRightParen,
}

// This function ensures that the current where stack element is correct based on the previous.
// Raises parser errors for strings like `WHERE NOT AND 1 = 1`, `WHERE 1 = 1 2 = 2`, or `WHERE ()`.
pub fn next_expected_token_from_current(expected_token: &WhereClauseExpectedNextToken, where_stack_element: &WhereStackElement, parser: &mut Parser) -> Result<WhereClauseExpectedNextToken, String> {
    match where_stack_element {
        WhereStackElement::Condition(_) => {
            if *expected_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                return Err(parser.format_error_nearby());
            }
            Ok(WhereClauseExpectedNextToken::LogicalOperatorRightParen)
        },
        WhereStackElement::LogicalOperator(logical_operator) => {
            match logical_operator {
                LogicalOperator::Not => {
                    if *expected_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                        return Err(parser.format_error_nearby());
                    }
                    Ok(WhereClauseExpectedNextToken::ConditionLeftParenNot)
                },
                _ => {
                    if *expected_token != WhereClauseExpectedNextToken::LogicalOperatorRightParen {
                        return Err(parser.format_error_nearby());
                    }
                    Ok(WhereClauseExpectedNextToken::ConditionLeftParenNot)
                }
            }
        },
        WhereStackElement::Parentheses(parentheses) => {
            match parentheses {
                Parentheses::Left => {
                    if *expected_token != WhereClauseExpectedNextToken::ConditionLeftParenNot {
                        return Err(parser.format_error_nearby());
                    }
                    Ok(WhereClauseExpectedNextToken::ConditionLeftParenNot)
                },
                Parentheses::Right => {
                    if *expected_token != WhereClauseExpectedNextToken::LogicalOperatorRightParen {
                        return Err(parser.format_error_nearby());
                    }
                    Ok(WhereClauseExpectedNextToken::LogicalOperatorRightParen)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{WhereCondition, Operand, Operator, Value};

    #[test]
    fn handles_next_condition_being_condition() {
        let mut parser = Parser::new(vec![]);
        let where_stack_element = WhereStackElement::Condition(WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))});
        let expected_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_err());
        let expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_ok());
    }

    #[test]
    fn handles_next_condition_being_logical_operator() {
        let mut parser = Parser::new(vec![]);
        // Not operator
        let where_stack_element = WhereStackElement::LogicalOperator(LogicalOperator::Not);
        let expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_ok());
        let expected_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_err());

        // Other logical operator (I used AND but OR should be the same)
        let where_stack_element = WhereStackElement::LogicalOperator(LogicalOperator::And);
        let expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_err());
        let expected_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_ok());
    }

    #[test]
    fn handles_next_condition_being_parentheses() {
        let mut parser = Parser::new(vec![]);
        // Left parentheses
        let where_stack_element = WhereStackElement::Parentheses(Parentheses::Left);
        let expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_ok());
        let expected_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_err());

        // Right parentheses
        let where_stack_element = WhereStackElement::Parentheses(Parentheses::Right);
        let expected_token = WhereClauseExpectedNextToken::ConditionLeftParenNot;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_err());
        let expected_token = WhereClauseExpectedNextToken::LogicalOperatorRightParen;
        assert!(next_expected_token_from_current(&expected_token, &where_stack_element, &mut parser).is_ok());
    }
}