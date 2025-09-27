use crate::interpreter::{
    ast::{
        ExistenceCheck, LogicalOperator, MathOperator, Operator, OrderByDirection,
        SelectStatementColumn, SelectStatementTable, SelectableStack, SelectableStackElement,
        helpers::token::token_to_value, parser::Parser,
    },
    tokenizer::token::TokenTypes,
};
use std::cmp::Ordering;

// Returns an error if the current token does not match the given token type
pub fn expect_token_type(parser: &Parser, token_type: TokenTypes) -> Result<(), String> {
    let token = parser.current_token()?;
    if token.token_type != token_type {
        return Err(parser.format_error());
    }
    Ok(())
}

pub fn get_table_name(
    parser: &mut Parser,
    allow_alias: bool,
) -> Result<SelectStatementTable, String> {
    expect_token_type(parser, TokenTypes::Identifier)?;
    let table_name = parser.current_token()?.value.to_string();
    parser.advance()?;
    if allow_alias && parser.current_token()?.token_type == TokenTypes::As {
        parser.advance()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        let alias = parser.current_token()?.value.to_string();
        parser.advance()?;
        return Ok(SelectStatementTable {
            table_name: table_name,
            alias: Some(alias),
        });
    }
    Ok(SelectStatementTable::new(table_name))
}

pub const TOKENS_NEEDING_SPECIAL_HANDLING: [TokenTypes; 5] = [
    TokenTypes::From,
    TokenTypes::SemiColon,
    TokenTypes::Where,
    TokenTypes::Order,
    TokenTypes::Limit,
];

pub fn get_selectables(
    parser: &mut Parser,
    allow_multiple: bool,
    order_by_directions: &mut Option<&mut Vec<OrderByDirection>>,
    selectable_names: &mut Option<&mut Vec<SelectStatementColumn>>,
) -> Result<SelectableStack, String> {
    #[derive(PartialEq)]
    enum ExtendedSelectableStackElement {
        SelectableStackElement(SelectableStackElement),
        LeftParen,
    }
    let mut output: Vec<SelectableStackElement> = vec![];
    let mut operators: Vec<ExtendedSelectableStackElement> = vec![];
    let mut depth = 0;
    let mut current_name = "".to_string();
    let mut current_alias: Option<String> = None;
    let mut current_table_name: Option<String> = None;

    let mut first = true;
    let mut expect_new_value = false; // Will be set after a valid ASC or DESC to ensure proper syntax
    loop {
        let last_token_type = parser.current_token()?.token_type.clone();

        if !first {
            parser.advance()?;
        }
        let was_first = first;
        first = false;

        let token = parser.current_token()?;

        // Tokens needing special handling
        // TODO: more tokens should be added here (e.g. Group for GROUP BY)
        if TOKENS_NEEDING_SPECIAL_HANDLING.contains(&token.token_type) {
            // Default ordering is ASC
            if !expect_new_value && let Some(order_by_directions_vector) = order_by_directions {
                order_by_directions_vector.push(OrderByDirection::Asc);
            }
            break;
        } else if expect_new_value && token.token_type != TokenTypes::Comma {
            return Err("Unexpected token after ordering direction".to_string());
        }

        if token.token_type == TokenTypes::Asterisk
            && (was_first || [TokenTypes::Comma, TokenTypes::LeftParen].contains(&last_token_type))
        {
            // * (All) is only allowed at certain places, otherwise it's * (Multiply)
            output.push(SelectableStackElement::All);
            current_name += token.value;
            continue;
        } else if token.token_type == TokenTypes::Comma {
            if depth == 0 {
                if !allow_multiple {
                    return Err("Unexpected token: COMMA".to_string());
                } else if let Some(selectable_names_vector) = selectable_names {
                    let mut column = SelectStatementColumn::new(current_name.clone());
                    column.alias = current_alias.clone();
                    column.table_name = current_table_name.clone();
                    selectable_names_vector.push(column);
                }
                // Default ordering is ASC
                if !expect_new_value && let Some(order_by_directions_vector) = order_by_directions {
                    order_by_directions_vector.push(OrderByDirection::Asc);
                }
                expect_new_value = false;
                current_name = "".to_string();
                current_alias = None;
            } else {
                current_name += token.value;
            }

            // Also push all current operators on the stack inside the current parenthesis
            while !operators.is_empty() {
                match operators.last() {
                    Some(value) => match value {
                        ExtendedSelectableStackElement::LeftParen => {
                            break;
                        }
                        ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                            output.push(inner.clone());
                            operators.pop();
                        }
                    },
                    None => {
                        break;
                    }
                }
            }
            continue;
        } else if token.token_type == TokenTypes::LeftParen {
            operators.push(ExtendedSelectableStackElement::LeftParen);
            current_name += token.value;
            depth += 1;
            continue;
        } else if token.token_type == TokenTypes::RightParen {
            depth -= 1;
            current_name += token.value;
            while let Some(operator) = operators.pop() {
                match operator {
                    ExtendedSelectableStackElement::LeftParen => {
                        break;
                    }
                    ExtendedSelectableStackElement::SelectableStackElement(value) => {
                        output.push(value);
                    }
                }
            }

            // If the top operator exists and owns the parenthesis, push it
            if operators.last().is_some() {
                match operators.last().unwrap() {
                    ExtendedSelectableStackElement::SelectableStackElement(inner) => match inner {
                        SelectableStackElement::Function(function) => {
                            if function.has_parentheses {
                                output.push(SelectableStackElement::Function(function.clone()));
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            };
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

        current_name += token.value;

        // Operators
        let operator = match token.token_type {
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
                                output.push(inner.clone());
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
            TokenTypes::Identifier => {
                let column = get_select_statement_column(parser)?;
                current_alias = column.alias.clone();
                current_table_name = column.table_name.clone();
                current_name = column.column_name.clone();
                SelectableStackElement::Column(column)
            }
            _ => return Err(parser.format_error()), // TODO: better error handling
        };
        output.push(element);
    }

    while !operators.is_empty() {
        match operators.pop() {
            Some(value) => match value {
                ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                    output.push(inner.clone());
                }
                _ => {}
            },
            _ => {}
        }
    }

    if let Some(selectable_names_vector) = selectable_names {
        let mut column = SelectStatementColumn::new(current_name);
        column.alias = current_alias;
        column.table_name = current_table_name;
        selectable_names_vector.push(column);
    }

    Ok(SelectableStack {
        selectables: output,
    })
}

pub fn exists_clause(
    parser: &mut Parser,
    check_type: ExistenceCheck,
) -> Result<Option<ExistenceCheck>, String> {
    if parser.current_token()?.token_type == TokenTypes::If {
        parser.advance()?;
        let token = parser.current_token()?;
        let existence_check = match (&token.token_type, check_type) {
            (TokenTypes::Not, ExistenceCheck::IfNotExists) => {
                parser.advance()?;
                expect_token_type(parser, TokenTypes::Exists)?;
                ExistenceCheck::IfNotExists
            }
            (TokenTypes::Exists, ExistenceCheck::IfExists) => ExistenceCheck::IfExists,
            (_, _) => return Err(parser.format_error()),
        };
        parser.advance()?;
        return Ok(Some(existence_check));
    }
    return Ok(None);
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

pub fn get_select_statement_column<'a>(
    parser: &mut Parser<'a>,
) -> Result<SelectStatementColumn, String> {
    let mut current_token = parser.current_token()?;
    let table_name = if parser.peek_token()?.token_type == TokenTypes::Dot {
        let table_name = current_token.value.to_string();
        parser.advance()?;
        parser.advance()?;
        current_token = parser.current_token()?;
        Some(table_name)
    } else {
        None
    };

    let column_name = current_token.value.to_string();
    let alias = if let Ok(peek_token) = parser.peek_token()
        && peek_token.token_type == TokenTypes::As
    {
        parser.advance()?;
        parser.advance()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        let alias = Some(parser.current_token()?.value.to_string());
        alias
    } else {
        None
    };
    let column = SelectStatementColumn {
        column_name: column_name,
        alias: alias,
        table_name: table_name,
    };
    Ok(column)
}

fn get_precedence(operator: &SelectableStackElement) -> Result<i32, String> {
    let result = match operator {
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

pub fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at {}: {}", i, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_handles_valid_hex_string() {
        let result = hex_decode("0A1A3F");
        assert!(result.is_ok());
        let expected = vec![0x0A, 0x1A, 0x3F];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn decode_handles_invalid_hex_string() {
        let result = hex_decode("0AZA3A");
        assert!(result.is_err());
        let expected = "Invalid hex at 2: invalid digit found in string";
        assert_eq!(expected, result.err().unwrap());

        let result = hex_decode("0A1");
        assert!(result.is_err());
        let expected = "Hex string must have even length";
        assert_eq!(expected, result.err().unwrap());
    }

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
    // TODO: add more tests
}
