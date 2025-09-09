use crate::interpreter::ast::helpers::order_by_clause::get_order_by;
use crate::interpreter::ast::helpers::limit_clause::get_limit;
use crate::interpreter::ast::{parser::Parser, SqlStatement, SelectStatementStack, SelectStatementStackElement, SetOperator, SelectStackOperators, SelectStatementColumns};
use crate::interpreter::ast::helpers::select_statement;
use crate::interpreter::ast::Parentheses;
use crate::interpreter::tokenizer::token::TokenTypes;

// Returns a SelectStatementStack which is an RPN representation of the SELECT statements and set operators.
pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    let mut statement_stack = SelectStatementStack {
        columns: SelectStatementColumns::All,
        elements: vec![],
        order_by_clause: None,
        limit_clause: None,
    };
    let mut columns = None;
    let mut set_operator_stack: Vec<SelectStackOperators> = vec![];

    loop {
        let token = parser.current_token()?;
        match token.token_type {
            TokenTypes::Select => {
                let mut statement = select_statement::get_statement(parser)?;
                columns = match columns {
                    None => Some(statement.columns.clone()),
                    Some(columns) => {
                        if statement.columns != columns {
                            return Err("Columns mismatch between SELECT statements in Union".to_string());
                        }
                        Some(columns)
                    },
                };
                if parser.current_token()?.token_type != TokenTypes::SemiColon {
                    if statement.order_by_clause.is_some() || statement.limit_clause.is_some() {
                        return Err("ORDER BY, or LIMIT clause not allowed with UNION SELECT statements".to_string());
                    }
                }
                else if statement_stack.elements.len() > 0 && parser.current_token()?.token_type == TokenTypes::SemiColon {
                    statement_stack.order_by_clause = statement.order_by_clause.take();
                    statement_stack.limit_clause = statement.limit_clause.take();
                }
                statement_stack.elements.push(SelectStatementStackElement::SelectStatement(statement));

            }
            TokenTypes::LeftParen => {
                set_operator_stack.push(SelectStackOperators::Parentheses(Parentheses::Left));
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::Parentheses(_) = current_set_operator {
                        break;
                    }
                    else if let SelectStackOperators::SetOperator(set_operator) = current_set_operator {
                        statement_stack.elements.push(SelectStatementStackElement::SetOperator(set_operator));
                    }
                    else {
                        return Err("Mismatched parentheses found.".to_string());
                    }
                }
                parser.advance()?;
                match parser.current_token()?.token_type {
                    TokenTypes::Order => {
                        statement_stack.order_by_clause = get_order_by(parser)?;
                        statement_stack.limit_clause = get_limit(parser)?;
                    }
                    TokenTypes::Limit => {
                        statement_stack.limit_clause = get_limit(parser)?;
                    }
                    _ => {},
                }
            }
            TokenTypes::Union | TokenTypes::Except => {
                let set_operator = get_set_operator(parser)?;
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::Parentheses(parentheses) = current_set_operator {
                        set_operator_stack.push(SelectStackOperators::Parentheses(parentheses));
                        break;
                    }
                    else if let SelectStackOperators::SetOperator(current_set_operator) = current_set_operator {
                        statement_stack.elements.push(SelectStatementStackElement::SetOperator(current_set_operator));
                    }
                }
                set_operator_stack.push(SelectStackOperators::SetOperator(set_operator));
            }
            TokenTypes::Intersect => {
                let set_operator = get_set_operator(parser)?;
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::SetOperator(current_set_operator) = current_set_operator {
                        if set_operator.is_greater_precedence(&current_set_operator) {
                            set_operator_stack.push(SelectStackOperators::SetOperator(current_set_operator));
                            break;
                        }
                        else {
                            statement_stack.elements.push(SelectStatementStackElement::SetOperator(current_set_operator));
                        }
                    }
                    else {
                        set_operator_stack.push(current_set_operator);
                        break;
                    }
                }
                set_operator_stack.push(SelectStackOperators::SetOperator(set_operator));
            }
            TokenTypes::SemiColon => break,
            _ => return Err(parser.format_error()),
        }
    }

    while let Some(current_set_operator) = set_operator_stack.pop() {
        if let SelectStackOperators::SetOperator(set_operator) = current_set_operator {
            statement_stack.elements.push(SelectStatementStackElement::SetOperator(set_operator));
        }
        else {
            return Err("Mismatched parentheses found.".to_string());
        }
    }
    match columns {
        Some(columns) => statement_stack.columns = columns,
        None => return Err("Error parsing SELECT statement. Columns not found.".to_string()),
    }
    return Ok(SqlStatement::Select(statement_stack));
}

fn get_set_operator(parser: &mut Parser) -> Result<SetOperator, String> {
    let token = parser.current_token()?;
    let set_operator = match token.token_type {
        TokenTypes::Union => {
            if parser.peek_token()?.token_type == TokenTypes::All {
                parser.advance()?;
                Ok(SetOperator::UnionAll)
            } else {
                Ok(SetOperator::Union)
            }
        },
        TokenTypes::Except => {
            Ok(SetOperator::Except)
        },
        TokenTypes::Intersect => {
            Ok(SetOperator::Intersect)
        },
        _ => Err("Expected token type: Union, Except, or Intersect was not found".to_string()),
    };
    parser.advance()?;
    return set_operator;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::SelectStatement;
    use crate::interpreter::ast::SetOperator;
    use crate::interpreter::ast::SelectStatementColumns;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::Operator;
    use crate::db::table::Value;
    use crate::interpreter::tokenizer::token::TokenTypes;
    use crate::interpreter::tokenizer::scanner::Token;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::SelectMode;

    fn simple_select_statement_tokens(id: &'static str) -> Vec<Token<'static>> {
        vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, id),
        ]
    }

    fn expected_simple_select_statement(id: i64) -> SelectStatementStackElement {
        SelectStatementStackElement::SelectStatement(SelectStatement {
            table_name: "users".to_string(),
            mode: SelectMode::All,
            columns: SelectStatementColumns::All,
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("id".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Integer(id)),
            })]),
            order_by_clause: None,
            limit_clause: None,
        })
    }


    #[test]
    fn simple_select_statement_is_generated_correctly() {
        // SELECT * FROM users WHERE id = 1;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![expected_simple_select_statement(1)],
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_set_operator_is_generated_correctly() {
        // SELECT * FROM users WHERE id = 1 UNION ALL SELECT * FROM users WHERE id = 2;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION"), token(TokenTypes::All, "ALL")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
            ],
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_multiple_set_operators_is_generated_correctly() {
        // SELECT 1 ... UNION ALL SELECT 2 ... INTERSECT SELECT 3 ... EXCEPT SELECT 4 ...;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::Intersect, "INTERSECT")]);
        tokens.append(&mut simple_select_statement_tokens("3"));
        tokens.append(&mut vec![token(TokenTypes::Except, "EXCEPT")]);
        tokens.append(&mut simple_select_statement_tokens("4"));
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                expected_simple_select_statement(3),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
                SelectStatementStackElement::SetOperator(SetOperator::Union),
                expected_simple_select_statement(4),
                SelectStatementStackElement::SetOperator(SetOperator::Except),
            ],
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_multiple_set_operators_and_parentheses_is_generated_correctly() {
        // (SELECT 1 ... UNION ALL SELECT 2 ...) INTERSECT (SELECT 3 ... EXCEPT SELECT 4 ...);
        let mut tokens = vec![token(TokenTypes::LeftParen, "(")];
        tokens.append(&mut simple_select_statement_tokens("1"));
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION")]);
        tokens.append(&mut vec![token(TokenTypes::All, "ALL")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::Intersect, "INTERSECT")]);
        tokens.append(&mut vec![token(TokenTypes::LeftParen, "(")]);
        tokens.append(&mut simple_select_statement_tokens("3"));
        tokens.append(&mut vec![token(TokenTypes::Except, "EXCEPT")]);
        tokens.append(&mut simple_select_statement_tokens("4"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
                expected_simple_select_statement(3),
                expected_simple_select_statement(4),
                SelectStatementStackElement::SetOperator(SetOperator::Except),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            ],
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_stack_with_all_clauses_is_generated_correctly() {
        // SELECT name FROM employees WHERE name = 'Henry' UNION ALL SELECT name FROM employees WHERE name = 'John' ORDER BY name LIMIT 10 OFFSET 15;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "employees"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "Henry"),
            token(TokenTypes::Union, "UNION"),
            token(TokenTypes::All, "ALL"),
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "employees"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "John"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "15"),
            token(TokenTypes::SemiColon, ";")
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::Specific(vec!["name".to_string()]),
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "employees".to_string(),
                    mode: SelectMode::All,
                    columns: SelectStatementColumns::Specific(vec!["name".to_string()]),
                    where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                        l_side: Operand::Identifier("name".to_string()),
                        operator: Operator::Equals,
                        r_side: Operand::Value(Value::Text("Henry".to_string())),
                    })]),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "employees".to_string(),
                    mode: SelectMode::All,
                    columns: SelectStatementColumns::Specific(vec!["name".to_string()]),
                    where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                        l_side: Operand::Identifier("name".to_string()),
                        operator: Operator::Equals,
                        r_side: Operand::Value(Value::Text("John".to_string())),
                    })]),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
            ],
            order_by_clause: Some(vec![OrderByClause {
                column: "name".to_string(),
                direction: OrderByDirection::Asc,
            }]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(15)),
            }),
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_order_by_and_parentheses_is_generated_correctly() {
        // (SELECT A UNION ALL SELECT B) ORDER BY name LIMIT 10 OFFSET 15;
        let mut tokens = vec![token(TokenTypes::LeftParen, "(")];
        tokens.append(&mut simple_select_statement_tokens("1"));
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION")]);
        tokens.append(&mut vec![token(TokenTypes::All, "ALL")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::Order, "ORDER")]);
        tokens.append(&mut vec![token(TokenTypes::By, "BY")]);
        tokens.append(&mut vec![token(TokenTypes::Identifier, "name")]);
        tokens.append(&mut vec![token(TokenTypes::Limit, "LIMIT")]);
        tokens.append(&mut vec![token(TokenTypes::IntLiteral, "10")]);
        tokens.append(&mut vec![token(TokenTypes::Offset, "OFFSET")]);
        tokens.append(&mut vec![token(TokenTypes::IntLiteral, "15")]);
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
            ],
            order_by_clause: Some(vec![OrderByClause {
                column: "name".to_string(),
                direction: OrderByDirection::Asc,
            }]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(15)),
            }),
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_intersect_with_limit_clause_and_parentheses_is_generated_correctly() {
        // (SELECT A INTERSECT SELECT B) LIMIT 10 OFFSET 15;
        let mut tokens = vec![token(TokenTypes::LeftParen, "(")];
        tokens.append(&mut simple_select_statement_tokens("1"));
        tokens.append(&mut vec![token(TokenTypes::Intersect, "INTERSECT")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::Limit, "LIMIT")]);
        tokens.append(&mut vec![token(TokenTypes::IntLiteral, "10")]);
        tokens.append(&mut vec![token(TokenTypes::Offset, "OFFSET")]);
        tokens.append(&mut vec![token(TokenTypes::IntLiteral, "15")]);
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            columns: SelectStatementColumns::All,
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            ],
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(15)),
            }),
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_columns_mismatch_is_generated_correctly() {
        // SELECT id, name FROM users UNION SELECT name FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Union, "UNION"),
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";")
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Columns mismatch between SELECT statements in Union".to_string());
    }
}