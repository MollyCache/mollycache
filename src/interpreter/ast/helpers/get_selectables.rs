use crate::interpreter::{
    ast::{
        FunctionName, FunctionSignature, LogicalOperator, MathOperator, Operator, OrderByDirection,
        SelectableColumn, SelectableStackElement, helpers::token::token_to_value, parser::Parser,
    },
    tokenizer::token::TokenTypes,
};
use std::cmp::Ordering;

#[derive(PartialEq)]
enum ExtendedSelectableStackElement {
    SelectableStackElement(SelectableStackElement),
    LeftParen,
}

pub fn get_selectables(
    parser: &mut Parser,
    allow_multiple: bool,
    allow_aliases: bool,
    order_by_directions: &mut Option<&mut Vec<OrderByDirection>>,
) -> Result<Vec<SelectableColumn>, String> {
    let mut all_columns: Vec<SelectableColumn> = vec![];
    let mut current_column: Vec<SelectableStackElement> = vec![];
    let mut current_name = "".to_string();
    let mut operators: Vec<ExtendedSelectableStackElement> = vec![];
    let mut depth = 0;

    let mut first = true;
    let mut expect_new_value = false; // Will be set after a valid ASC or DESC (if ORDER BY) or after a valid AS <identifier> (if SELECT) to ensure proper syntax
    let mut expect_alias = false; // Will be set after a valid AS to ensure proper syntax
    loop {
        let last_token_type = parser.current_token()?.token_type.clone();

        if !first {
            parser.advance()?;
        }
        let was_first = first;
        first = false;

        let token = parser.current_token()?;

        if expect_alias {
            if token.token_type == TokenTypes::Identifier {
                expect_new_value = true;
                expect_alias = false;
                current_name = token.value.to_string();
                continue;
            } else {
                return Err(parser.format_error());
            }
        } else if [
            TokenTypes::From,
            TokenTypes::SemiColon,
            TokenTypes::Where,
            TokenTypes::Order,
            TokenTypes::Limit,
            TokenTypes::Union,
            TokenTypes::Intersect,
            TokenTypes::Except,
            TokenTypes::EOF,
        ]
        .contains(&token.token_type)
        {
            // Tokens needing special handling
            // TODO: more tokens should be added here (e.g. Group for GROUP BY)
            // Default ordering is ASC
            if !expect_new_value && let Some(order_by_directions_vector) = order_by_directions {
                order_by_directions_vector.push(OrderByDirection::Asc);
            }
            break;
        } else if expect_new_value && token.token_type != TokenTypes::Comma {
            return Err("Unexpected token after ordering direction".to_string());
        } else if token.token_type == TokenTypes::RightParen && depth == 0 {
            // When deadling with set operators, a SELECT statement may end with ) (so a WHERE statement may too)
            break;
        }

        if token.token_type == TokenTypes::Asterisk
            && (was_first || [TokenTypes::Comma, TokenTypes::LeftParen].contains(&last_token_type))
        {
            // * (All) is only allowed at certain places, otherwise it's * (Multiply)
            current_column.push(SelectableStackElement::All);
            current_name += token.value;
            current_name += " ";
            continue;
        } else if token.token_type == TokenTypes::Comma {
            // Push all current operators on the stack inside the current parenthesis
            while !operators.is_empty() {
                match operators.last() {
                    Some(value) => match value {
                        ExtendedSelectableStackElement::LeftParen => {
                            break;
                        }
                        ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                            current_column.push(inner.clone());
                            operators.pop();
                        }
                    },
                    None => {
                        break;
                    }
                }
            }

            if depth == 0 {
                if !allow_multiple || current_column.len() == 0 {
                    return Err("Unexpected token: COMMA".to_string());
                }

                // Default ordering is ASC
                if !expect_new_value && let Some(order_by_directions_vector) = order_by_directions {
                    order_by_directions_vector.push(OrderByDirection::Asc);
                }

                current_name = current_name.trim().to_string();
                all_columns.push(SelectableColumn {
                    selectables: current_column,
                    column_name: current_name,
                });

                expect_new_value = false;
                current_column = vec![];
                current_name = "".to_string();
            } else {
                current_name += token.value;
                current_name += " ";
            }

            continue;
        } else if token.token_type == TokenTypes::LeftParen {
            operators.push(ExtendedSelectableStackElement::LeftParen);
            current_name += token.value;
            current_name += " ";
            depth += 1;
            continue;
        } else if token.token_type == TokenTypes::RightParen {
            depth -= 1;
            current_name += token.value;
            current_name += " ";
            while let Some(operator) = operators.pop() {
                match operator {
                    ExtendedSelectableStackElement::LeftParen => {
                        break;
                    }
                    ExtendedSelectableStackElement::SelectableStackElement(value) => {
                        current_column.push(value);
                    }
                }
            }

            // If the top operator exists and owns the parenthesis, push it
            let mut is_function = false;
            if let Some(last) = operators.last() {
                match last {
                    ExtendedSelectableStackElement::SelectableStackElement(inner) => match inner {
                        SelectableStackElement::Function(function) => {
                            if function.has_parentheses {
                                is_function = true;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            if is_function {
                if let Some(ExtendedSelectableStackElement::SelectableStackElement(elem)) =
                    operators.pop()
                {
                    current_column.push(elem);
                }
            }
            continue;
        } else if token.token_type == TokenTypes::As {
            if depth != 0 || !allow_aliases {
                return Err("Unexpected token: AS".to_string());
            }
            expect_alias = true;
            continue;
        }

        // Handle ASC and DESC if order_by_directions is set
        if let Some(order_by_directions_vector) = order_by_directions {
            let found = match token.token_type {
                TokenTypes::Asc => Some(OrderByDirection::Asc),
                TokenTypes::Desc => Some(OrderByDirection::Desc),
                _ => None,
            };
            if found.is_some() && depth != 0 {
                return Err("Found unexpected ordering token".to_string());
            } else if let Some(order) = found {
                expect_new_value = true;
                order_by_directions_vector.push(order);
                continue;
            }
        }

        // Update current_name
        match token.token_type {
            TokenTypes::StringLiteral => current_name.push_str(&format!("'{}'", token.value)),
            _ => current_name += token.value,
        };
        current_name += " ";

        // Operators
        let operator = match token.token_type {
            // Functions
            TokenTypes::Count => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Count,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::Sum => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Sum,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::Avg => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Avg,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::Min => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Min,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::Max => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Max,
                input_count: 1,
                has_parentheses: true,
            })),
            // TODO: Expand time and date functions to work with modifiers.
            TokenTypes::Date => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Date,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::Time => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::Time,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::DateTime => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::DateTime,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::JulianDay => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::JulianDay,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::UnixEpoch => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::UnixEpoch,
                input_count: 1,
                has_parentheses: true,
            })),
            TokenTypes::TimeDiff => Some(SelectableStackElement::Function(FunctionSignature {
                name: FunctionName::TimeDiff,
                input_count: 1,
                has_parentheses: true,
            })),
            // Operators
            TokenTypes::Equals => Some(SelectableStackElement::Operator(Operator::Equals)),
            TokenTypes::NotEquals => Some(SelectableStackElement::Operator(Operator::NotEquals)),
            TokenTypes::LessThan => Some(SelectableStackElement::Operator(Operator::LessThan)),
            TokenTypes::GreaterThan => {
                Some(SelectableStackElement::Operator(Operator::GreaterThan))
            }
            TokenTypes::LessEquals => Some(SelectableStackElement::Operator(Operator::LessEquals)),
            TokenTypes::GreaterEquals => {
                Some(SelectableStackElement::Operator(Operator::GreaterEquals))
            }
            TokenTypes::In => Some(SelectableStackElement::Operator(Operator::In)),
            // TODO: handle NOT IN (not a token)
            TokenTypes::Is => Some(SelectableStackElement::Operator(Operator::Is)),
            // TODO: handle IS NOT (not a token)
            // Logical operators
            TokenTypes::Not => Some(SelectableStackElement::LogicalOperator(
                LogicalOperator::Not,
            )),
            TokenTypes::And => Some(SelectableStackElement::LogicalOperator(
                LogicalOperator::And,
            )),
            TokenTypes::Or => Some(SelectableStackElement::LogicalOperator(LogicalOperator::Or)),
            // Math operators
            TokenTypes::Plus => Some(SelectableStackElement::MathOperator(MathOperator::Add)),
            TokenTypes::Minus => Some(SelectableStackElement::MathOperator(MathOperator::Subtract)),
            TokenTypes::Asterisk => {
                Some(SelectableStackElement::MathOperator(MathOperator::Multiply))
            }
            TokenTypes::Divide => Some(SelectableStackElement::MathOperator(MathOperator::Divide)),
            TokenTypes::Modulo => Some(SelectableStackElement::MathOperator(MathOperator::Modulo)),
            _ => None,
        };

        if let Some(value) = operator {
            while operators.len() > 0 {
                match operators.last() {
                    Some(last) => match last {
                        ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                            if compare_precedence(&value, inner)? != Ordering::Greater {
                                current_column.push(inner.clone());
                                operators.pop();
                            } else {
                                break;
                            }
                        }
                        _ => {
                            break;
                        }
                    },
                    None => {
                        break;
                    }
                }
            }

            operators.push(ExtendedSelectableStackElement::SelectableStackElement(
                value,
            ));
            continue;
        }

        // Tokens that are automatically added to output
        let element = match token.token_type {
            // All
            TokenTypes::All => SelectableStackElement::All,
            // Literals
            TokenTypes::IntLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::RealLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::StringLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::HexLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::Null => SelectableStackElement::Value(token_to_value(parser)?),
            // TODO: handle ValueList (arrays)
            TokenTypes::Identifier => SelectableStackElement::Column(token.value.to_string()), // TODO: verify it's a column, AND handle multi-tokens columns with AS (table_name.column_name)
            _ => return Err(parser.format_error()), // TODO: better error handling
        };
        current_column.push(element);
    }

    while !operators.is_empty() {
        match operators.pop() {
            Some(value) => match value {
                ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                    current_column.push(inner.clone());
                }
                _ => {}
            },
            _ => {}
        }
    }

    if current_column.len() > 0 {
        current_name = current_name.trim().to_string();
        all_columns.push(SelectableColumn {
            selectables: current_column,
            column_name: current_name,
        });
    }

    Ok(all_columns)
}

pub fn compare_precedence(
    first: &SelectableStackElement,
    second: &SelectableStackElement,
) -> Result<Ordering, String> {
    let first_precedence = get_precedence(first)?;
    let second_precedence = get_precedence(second)?;
    return if first_precedence == second_precedence {
        Ok(Ordering::Equal)
    } else if first_precedence < second_precedence {
        Ok(Ordering::Less)
    } else {
        Ok(Ordering::Greater)
    };
}

fn get_precedence(operator: &SelectableStackElement) -> Result<i32, String> {
    let result = match operator {
        SelectableStackElement::Function(_) => 50,
        SelectableStackElement::MathOperator(MathOperator::Multiply) => 40,
        SelectableStackElement::MathOperator(MathOperator::Divide) => 40,
        SelectableStackElement::MathOperator(MathOperator::Modulo) => 40,

        SelectableStackElement::MathOperator(MathOperator::Add) => 35,
        SelectableStackElement::MathOperator(MathOperator::Subtract) => 35,

        SelectableStackElement::Operator(Operator::GreaterThan) => 30,
        SelectableStackElement::Operator(Operator::LessThan) => 30,
        SelectableStackElement::Operator(Operator::GreaterEquals) => 30,
        SelectableStackElement::Operator(Operator::LessEquals) => 30,

        SelectableStackElement::Operator(Operator::Equals) => 25,
        SelectableStackElement::Operator(Operator::NotEquals) => 25,
        SelectableStackElement::Operator(Operator::Is) => 25,
        SelectableStackElement::Operator(Operator::IsNot) => 25,
        SelectableStackElement::Operator(Operator::In) => 25,
        SelectableStackElement::Operator(Operator::NotIn) => 25,

        SelectableStackElement::LogicalOperator(LogicalOperator::Not) => 20,
        SelectableStackElement::LogicalOperator(LogicalOperator::And) => 15,
        SelectableStackElement::LogicalOperator(LogicalOperator::Or) => 10,
        _ => return Err("Not an operator".to_string()), // TODO: better error message
    };

    return Ok(result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{FunctionName, LogicalOperator, MathOperator, SelectableStackElement};

    #[test]
    fn precedence_handles_correctly() {
        let result = compare_precedence(
            &SelectableStackElement::MathOperator(MathOperator::Multiply),
            &SelectableStackElement::MathOperator(MathOperator::Add),
        );
        assert!(result.is_ok());
        assert_eq!(Ordering::Greater, result.unwrap());
        let result = compare_precedence(
            &SelectableStackElement::MathOperator(MathOperator::Add),
            &SelectableStackElement::MathOperator(MathOperator::Multiply),
        );
        assert!(result.is_ok());
        assert_eq!(Ordering::Less, result.unwrap());
        let result = compare_precedence(
            &SelectableStackElement::LogicalOperator(LogicalOperator::And),
            &SelectableStackElement::LogicalOperator(LogicalOperator::Or),
        );
        assert!(result.is_ok());
        assert_eq!(Ordering::Greater, result.unwrap());
        let result = compare_precedence(
            &SelectableStackElement::LogicalOperator(LogicalOperator::Not),
            &SelectableStackElement::LogicalOperator(LogicalOperator::And),
        );
        assert!(result.is_ok());
        assert_eq!(Ordering::Greater, result.unwrap());
    }

    #[test]
    fn get_selectables_parses_functions_correctly() {
        use crate::interpreter::ast::parser::Parser;
        use crate::interpreter::ast::test_utils::token;
        use crate::interpreter::tokenizer::token::TokenTypes;

        let tokens = vec![
            token(TokenTypes::Count, "COUNT"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Sum, "SUM"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "salary"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::From, "FROM"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_selectables(&mut parser, true, true, &mut None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let columns = result.unwrap();
        assert_eq!(columns.len(), 2);

        // Check COUNT(*)
        assert_eq!(columns[0].selectables.len(), 2); // All, Count
        match &columns[0].selectables[1] {
            SelectableStackElement::Function(f) => {
                assert_eq!(f.name, FunctionName::Count);
                assert_eq!(f.input_count, 1);
            }
            _ => panic!("Expected Count function"),
        }

        // Check SUM(salary)
        assert_eq!(columns[1].selectables.len(), 2); // Column, Sum
        match &columns[1].selectables[1] {
            SelectableStackElement::Function(f) => {
                assert_eq!(f.name, FunctionName::Sum);
                assert_eq!(f.input_count, 1);
            }
            _ => panic!("Expected Sum function"),
        }
    }

    #[test]
    fn get_selectables_works_with_date_and_time_functions() {
        // TODO: Add tests for date and time functions
    }
}

