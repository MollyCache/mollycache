use crate::{interpreter::{
    ast::{
        parser::Parser, SelectStatement, SelectableStack, FunctionName, FunctionSignature, Operator, LogicalOperator, MathOperator, SelectableStackElement, WhereStackElement,
        helpers::{
            common::{get_table_name, expect_token_type, token_to_value, compare_precedence},
            order_by_clause::get_order_by, where_stack::get_where_clause, limit_clause::get_limit
        }
    }, 
    tokenizer::token::TokenTypes,
}};

pub fn get_statement(parser: &mut Parser) -> Result<SelectStatement, String> {
    parser.advance()?;
    let (columns, column_names) = get_columns_and_names(parser)?;
    expect_token_type(parser, TokenTypes::From)?; // TODO: this is not true, you can do SELECT 1;
    parser.advance()?;
    let table_name = get_table_name(parser)?;
    let where_clause: Option<Vec<WhereStackElement>> = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;
    
    return Ok(SelectStatement {
            table_name: table_name,
            columns: columns,
            column_names: column_names,
            where_clause: where_clause,
            order_by_clause: order_by_clause,
            limit_clause: limit_clause,
    });
}

fn get_columns_and_names(parser: &mut Parser) -> Result<(SelectableStack, Vec<String>), String> {
    #[derive(PartialEq)]
    enum ExtendedSelectableStackElement {
        SelectableStackElement(SelectableStackElement),
        LeftParen
    }
    let mut output: Vec<SelectableStackElement> = vec![];
    let mut operators: Vec<ExtendedSelectableStackElement> = vec![];
    let mut depth = 0;
    let mut column_names: Vec<String> = vec![];
    let mut current_name = "".to_string();

    let mut first = true;
    loop {
        let last_token_type = parser.current_token()?.token_type.clone();

        if !first { parser.advance()?; }
        let was_first = first;
        first = false;

        let token = parser.current_token()?;

        // Tokens needing special handling
        if [TokenTypes::From, TokenTypes::SemiColon].contains(&token.token_type) {
            break;
        } else if token.token_type == TokenTypes::Asterisk && (was_first || [TokenTypes::Comma, TokenTypes::LeftParen].contains(&last_token_type)) {
            // * (All) is only allowed at certain places, otherwise it's * (Multiply)
            output.push(SelectableStackElement::All);
            current_name += token.value;
            continue;
        } else if token.token_type == TokenTypes::Comma {
            if depth == 0 {
                column_names.push(current_name);
                current_name = "".to_string();
            } else {
                current_name += token.value;
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
                    },
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
                        },
                        _ => {},
                    },
                    _ => {},
                }
            };
            continue;
        }

        current_name += token.value;

        // Operators
        let operator = match token.token_type {
            // Operators
            TokenTypes::Equals => Some(SelectableStackElement::Operator(Operator::Equals)),
            TokenTypes::NotEquals => Some(SelectableStackElement::Operator(Operator::NotEquals)),
            TokenTypes::LessThan => Some(SelectableStackElement::Operator(Operator::LessThan)),
            TokenTypes::GreaterThan => Some(SelectableStackElement::Operator(Operator::GreaterThan)),
            TokenTypes::LessEquals => Some(SelectableStackElement::Operator(Operator::LessEquals)),
            TokenTypes::GreaterEquals => Some(SelectableStackElement::Operator(Operator::GreaterEquals)),
            TokenTypes::In => Some(SelectableStackElement::Operator(Operator::In)),
            // TODO: handle NOT IN (not a token)
            TokenTypes::Is => Some(SelectableStackElement::Operator(Operator::Is)),
            // TODO: handle IS NOT (not a token)
            // Logical operators
            TokenTypes::Not => Some(SelectableStackElement::LogicalOperator(LogicalOperator::Not)),
            TokenTypes::And => Some(SelectableStackElement::LogicalOperator(LogicalOperator::And)),
            TokenTypes::Or => Some(SelectableStackElement::LogicalOperator(LogicalOperator::Or)),
            // Math operators
            TokenTypes::Plus => Some(SelectableStackElement::MathOperator(MathOperator::Add)),
            TokenTypes::Minus => Some(SelectableStackElement::MathOperator(MathOperator::Subtract)),
            TokenTypes::Asterisk => Some(SelectableStackElement::MathOperator(MathOperator::Multiply)),
            TokenTypes::Divide => Some(SelectableStackElement::MathOperator(MathOperator::Divide)),
            TokenTypes::Modulo => Some(SelectableStackElement::MathOperator(MathOperator::Modulo)),
            _ => None,
        };

        if let Some(value) = operator {
            while operators.len() > 0 {
                match operators.last() {
                    Some(last) => match last {
                        ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                            if compare_precedence(&value, inner)? > 0 {
                                output.push(inner.clone());
                                operators.pop();
                            } else {
                                break;
                            }
                        },
                        _ => { break; }
                    }
                    None => { break; }
                }
            }

            operators.push(ExtendedSelectableStackElement::SelectableStackElement(value));
            continue;
        }

        // Tokens that are automatically added to output
        let element = match token.token_type {
            // All
            TokenTypes::All => SelectableStackElement::All,
            // Literals
            TokenTypes::IntLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::RealLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::String => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::HexLiteral => SelectableStackElement::Value(token_to_value(parser)?),
            TokenTypes::Null => SelectableStackElement::Value(token_to_value(parser)?),
            // TODO: handle ValueList (arrays)
            TokenTypes::Identifier => SelectableStackElement::Column(token.value.to_string()), // TODO: verify it's a column, AND handle multi-tokens columns with AS (table_name.column_name)
            _ => { return Err("Unexpected identifier".to_string()) } // TODO: better error handling
        };
        output.push(element);
    }

    while !operators.is_empty() {
        match operators.last() {
            Some(value) => match value {
                ExtendedSelectableStackElement::SelectableStackElement(inner) => {
                    output.push(inner.clone());
                },
                _ => {}
            }
            _ => {}
        }
    }

    column_names.push(current_name);

    Ok((SelectableStack { selectables: output }, column_names))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::Operator;
    use crate::db::table::Value;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::SelectableStackElement;

    #[test]
    fn select_statement_with_all_tokens_is_generated_correctly() {
        // SELECT * FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "users".to_string(),
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec!["*".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
    }

    #[test]
    fn select_statement_with_a_single_column_is_generated_correctly() {
        // SELECT id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "guests".to_string(),
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
            },
            column_names: vec!["id".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
    }

    #[test]
    fn select_statement_with_multiple_columns_is_generated_correctly() {
        // SELECT id, name FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "users".to_string(),
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Column("name".to_string()),
                ],
            },
            column_names: vec!["id".to_string(), "name".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
    }

    #[test]
    fn select_statement_with_all_clauses_is_generated_correctly() {
        // SELECT id FROM guests WHERE id = 1 ORDER BY id ASC, name DESC, age ASC LIMIT 10 OFFSET 5;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asc, "ASC"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Desc, "DESC"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "guests".to_string(),
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())]
            },
            column_names: vec!["id".to_string()],
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                }),
            ]),
            order_by_clause: Some(vec![
                OrderByClause {
                    column: "id".to_string(),
                    direction: OrderByDirection::Asc,
                },
                OrderByClause {
                    column: "name".to_string(),
                    direction: OrderByDirection::Desc,
                },
                OrderByClause {
                    column: "age".to_string(),
                    direction: OrderByDirection::Asc,
                }
            ]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(5)),
            }),
        };
        assert_eq!(expected, statement);
    }
}