use crate::interpreter::{
    ast::{
        SelectMode, SelectStatement, SelectStatementColumn, SelectableStack, WhereStackElement,
        helpers::{
            common::{get_selectables, get_table_name},
            limit_clause::get_limit,
            order_by_clause::get_order_by,
            token::expect_token_type,
            where_clause::get_where_clause,
        },
        parser::Parser,
    },
    tokenizer::token::TokenTypes,
};

pub fn get_statement(parser: &mut Parser) -> Result<SelectStatement, String> {
    parser.advance()?;
    let mode = match parser.current_token()?.token_type {
        TokenTypes::Distinct => {
            parser.advance()?;
            SelectMode::Distinct
        }
        _ => SelectMode::All,
    };
    let (columns, column_names) = get_columns_and_names(parser)?;
    expect_token_type(parser, TokenTypes::From)?; // TODO: this is not true, you can do SELECT 1;
    parser.advance()?;
    let table_name = get_table_name(parser, true)?;
    let where_clause: Option<Vec<WhereStackElement>> = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;

    return Ok(SelectStatement {
        table_name: table_name,
        mode: mode,
        columns: columns,
        column_names: column_names,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    });
}

fn get_columns_and_names(
    parser: &mut Parser,
) -> Result<(SelectableStack, Vec<SelectStatementColumn>), String> {
    let mut column_names: Vec<SelectStatementColumn> = vec![];
    Ok((
        get_selectables(parser, true, &mut None, &mut Some(&mut column_names))?,
        column_names,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::Value;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::Operator;
    use crate::interpreter::ast::MathOperator;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::SelectStatementTable;
    use crate::interpreter::ast::SelectableStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::test_utils::token;

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
        assert_eq!(
            statement,
            SelectStatement {
                table_name: SelectStatementTable::new("users".to_string()),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All],
                },
                column_names: vec![SelectStatementColumn::new("*".to_string())],
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }
        );
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
        assert_eq!(
            statement,
            SelectStatement {
                table_name: SelectStatementTable::new("guests".to_string()),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::Column(SelectStatementColumn::new(
                        "id".to_string()
                    ))],
                },
                column_names: vec![SelectStatementColumn::new("id".to_string())],
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }
        );
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
        assert_eq!(
            statement,
            SelectStatement {
                table_name: SelectStatementTable::new("users".to_string()),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![
                        SelectableStackElement::Column(SelectStatementColumn::new(
                            "id".to_string()
                        )),
                        SelectableStackElement::Column(SelectStatementColumn::new(
                            "name".to_string()
                        )),
                    ],
                },
                column_names: vec![
                    SelectStatementColumn::new("id".to_string()),
                    SelectStatementColumn::new("name".to_string())
                ],
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }
        );
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
            table_name: SelectStatementTable::new("guests".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column(SelectStatementColumn::new(
                    "id".to_string(),
                ))],
            },
            column_names: vec![SelectStatementColumn::new("id".to_string())],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("id".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Integer(1)),
            })]),
            order_by_clause: Some(OrderByClause {
                columns: SelectableStack {
                    selectables: vec![
                        SelectableStackElement::Column(SelectStatementColumn::new(
                            "id".to_string(),
                        )),
                        SelectableStackElement::Column(SelectStatementColumn::new(
                            "name".to_string(),
                        )),
                        SelectableStackElement::Column(SelectStatementColumn::new(
                            "age".to_string(),
                        )),
                    ],
                },
                column_names: vec![
                    SelectStatementColumn::new("id".to_string()),
                    SelectStatementColumn::new("name".to_string()),
                    SelectStatementColumn::new("age".to_string()),
                ],
                directions: vec![
                    OrderByDirection::Asc,
                    OrderByDirection::Desc,
                    OrderByDirection::Asc,
                ],
            }),
            limit_clause: Some(LimitClause {
                limit: 10,
                offset: Some(5),
            }),
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_distinct_mode_is_generated_correctly() {
        // SELECT DISTINCT id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Distinct, "DISTINCT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable::new("guests".to_string()),
            column_names: vec![SelectStatementColumn::new("id".to_string())],
            mode: SelectMode::Distinct,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column(SelectStatementColumn::new(
                    "id".to_string(),
                ))],
            },
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_column_alias_is_generated_correctly() {
        // SELECT id AS user_id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "user_id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable::new("guests".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column(SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("user_id".to_string()),
                    table_name: None,
                })],
            },
            column_names: vec![SelectStatementColumn {
                column_name: "id".to_string(),
                alias: Some("user_id".to_string()),
                table_name: None,
            }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_column_alias_and_table_name_is_generated_correctly() {
        // SELECT name as some, id2, id AS user_id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "some"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "id2"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "user_id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable::new("guests".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "name".to_string(),
                        alias: Some("some".to_string()),
                        table_name: None,
                    }),
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id2".to_string(),
                        alias: None,
                        table_name: None,
                    }),
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id".to_string(),
                        alias: Some("user_id".to_string()),
                        table_name: None,
                    }),
                ],
            },
            column_names: vec![
                SelectStatementColumn {
                    column_name: "name".to_string(),
                    alias: Some("some".to_string()),
                    table_name: None,
                },
                SelectStatementColumn {
                    column_name: "id2".to_string(),
                    alias: None,
                    table_name: None,
                },
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("user_id".to_string()),
                    table_name: None,
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_table_name_alias_is_generated_correctly() {
        // SELECT id FROM guests AS g;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "g"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable {
                table_name: "guests".to_string(),
                alias: Some("g".to_string()),
            },
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column(SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: None,
                    table_name: None,
                })],
            },
            column_names: vec![SelectStatementColumn {
                column_name: "id".to_string(),
                alias: None,
                table_name: None,
            }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_column_table_name_and_alias_is_generated_correctly() {
        // SELECT u.id as user_id, t.id as ticket_id FROM guests AS u;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "u"),
            token(TokenTypes::Dot, "."),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "user_id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "t"),
            token(TokenTypes::Dot, "."),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "ticket_id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "u"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable {
                table_name: "guests".to_string(),
                alias: Some("u".to_string()),
            },
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id".to_string(),
                        alias: Some("user_id".to_string()),
                        table_name: Some("u".to_string()),
                    }),
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id".to_string(),
                        alias: Some("ticket_id".to_string()),
                        table_name: Some("t".to_string()),
                    }),
                ],
            },
            column_names: vec![
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("user_id".to_string()),
                    table_name: Some("u".to_string()),
                },
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("ticket_id".to_string()),
                    table_name: Some("t".to_string()),
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_multiple_columns_and_table_name_alias_is_generated_correctly() {
        // SELECT u.id as user_id, plane, id as ticket_id FROM guests AS u;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "u"),
            token(TokenTypes::Dot, "."),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "user_id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "plane"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "ticket_id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::As, "AS"),
            token(TokenTypes::Identifier, "u"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: SelectStatementTable {
                table_name: "guests".to_string(),
                alias: Some("u".to_string()),
            },
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id".to_string(),
                        alias: Some("user_id".to_string()),
                        table_name: Some("u".to_string()),
                    }),
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "plane".to_string(),
                        alias: None,
                        table_name: None,
                    }),
                    SelectableStackElement::Column(SelectStatementColumn {
                        column_name: "id".to_string(),
                        alias: Some("ticket_id".to_string()),
                        table_name: None,
                    }),
                ],
            },
            column_names: vec![
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("user_id".to_string()),
                    table_name: Some("u".to_string()),
                },
                SelectStatementColumn {
                    column_name: "plane".to_string(),
                    alias: None,
                    table_name: None,
                },
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: Some("ticket_id".to_string()),
                    table_name: None,
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_complex_selectables_is_generated_correctly() {
        // SELECT id, age + money, 2 * ((age - (id % age - id / money))), money >= 300.0 OR NOT age > 20 AND money >= 100.5 FROM people ORDER BY id * age;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Distinct, "DISTINCT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::Plus, "+"),
            token(TokenTypes::Identifier, "money"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::IntLiteral, "2"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::Minus, "-"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Modulo, "%"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::Minus, "-"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Divide, "/"),
            token(TokenTypes::Identifier, "money"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "people"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();

        let select_statement_column_id = SelectStatementColumn {
            column_name: "id".to_string(),
            alias: None,
            table_name: None,
        };
        let select_statement_column_age = SelectStatementColumn {
            column_name: "age".to_string(),
            alias: None,
            table_name: None,
        };
        let select_statement_column_money = SelectStatementColumn {
            column_name: "money".to_string(),
            alias: None,
            table_name: None,
        };

        let expected = SelectStatement {
            table_name: SelectStatementTable {
                table_name: "people".to_string(),
                alias: None
            },
            mode: SelectMode::Distinct,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(select_statement_column_id.clone()),
                    SelectableStackElement::Column(select_statement_column_age.clone()),
                    SelectableStackElement::Column(select_statement_column_money.clone()),
                    SelectableStackElement::MathOperator(MathOperator::Add),
                    SelectableStackElement::Value(Value::Integer(2)),
                    SelectableStackElement::Column(select_statement_column_age.clone()),
                    SelectableStackElement::Column(select_statement_column_id.clone()),
                    SelectableStackElement::Column(select_statement_column_age.clone()),
                    SelectableStackElement::MathOperator(MathOperator::Modulo),
                    SelectableStackElement::Column(select_statement_column_id.clone()),
                    SelectableStackElement::MathOperator(MathOperator::Divide),
                    SelectableStackElement::MathOperator(MathOperator::Subtract),
                    SelectableStackElement::MathOperator(MathOperator::Subtract),
                    SelectableStackElement::MathOperator(MathOperator::Multiply),
                    SelectableStackElement::Column(select_statement_column_money.clone()),
                    SelectableStackElement::Operator(Operator::GreaterEquals),
                ],
            },
            column_names: vec![
                SelectStatementColumn {
                    column_name: "id".to_string(),
                    alias: None,
                    table_name: None,
                },
                SelectStatementColumn {
                    column_name: "age + money".to_string(),
                    alias: None,
                    table_name: None,
                },
                SelectStatementColumn {
                    column_name: "2 * ((age - (id % age - id / money)))".to_string(),
                    alias: None,
                    table_name: None,
                },
            ],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: SelectableStack {
                    selectables: vec![
                        SelectableStackElement::Column(select_statement_column_id.clone()),
                        SelectableStackElement::Column(select_statement_column_age.clone()),
                        SelectableStackElement::MathOperator(MathOperator::Multiply),
                    ],
                },
                column_names: vec![SelectStatementColumn {
                    column_name: "id * age".to_string(),
                    alias: None,
                    table_name: None,
                }],
                directions: vec![OrderByDirection::Asc],
            }),
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }
}
