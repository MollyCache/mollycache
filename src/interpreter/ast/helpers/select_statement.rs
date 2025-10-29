use crate::interpreter::{
    ast::{
        SelectMode, SelectStatement, SelectableColumn, TableAliases,
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
use std::collections::HashMap;

pub fn get_statement(parser: &mut Parser) -> Result<SelectStatement, String> {
    parser.advance()?;
    let mode = match parser.current_token()?.token_type {
        TokenTypes::Distinct => {
            parser.advance()?;
            SelectMode::Distinct
        }
        _ => SelectMode::All,
    };
    let columns = get_columns_and_names(parser)?;
    expect_token_type(parser, TokenTypes::From)?; // TODO: this is not true, you can do SELECT 1;
    parser.advance()?;
    let (table_name, table_alias) = get_table_name(parser)?;
    let mut aliases = HashMap::new();
    if table_alias != "" {
        aliases.insert(table_alias, table_name.clone());
    }
    let where_clause = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;

    return Ok(SelectStatement {
        table_name: table_name,
        table_aliases: TableAliases(aliases),
        mode: mode,
        columns: columns,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    });
}

fn get_columns_and_names(parser: &mut Parser) -> Result<Vec<SelectableColumn>, String> {
    Ok(get_selectables(parser, true, true, &mut None)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::Value;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::LogicalOperator;
    use crate::interpreter::ast::MathOperator;
    use crate::interpreter::ast::Operator;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::SelectableStackElement;
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
                table_name: "users".to_string(),
                table_aliases: TableAliases(HashMap::new()),
                mode: SelectMode::All,
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::All],
                    column_name: "*".to_string(),
                }],
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
                table_name: "guests".to_string(),
                table_aliases: TableAliases(HashMap::new()),
                mode: SelectMode::All,
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                }],
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
                table_name: "users".to_string(),
                table_aliases: TableAliases(HashMap::new()),
                mode: SelectMode::All,
                columns: vec![
                    SelectableColumn {
                        selectables: vec![SelectableStackElement::Column("id".to_string())],
                        column_name: "id".to_string(),
                    },
                    SelectableColumn {
                        selectables: vec![SelectableStackElement::Column("name".to_string())],
                        column_name: "name".to_string(),
                    },
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
            table_name: "guests".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
                column_name: "id".to_string(),
            }],
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(1)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "id = 1".to_string(),
            }),
            order_by_clause: Some(OrderByClause {
                columns: vec![
                    SelectableColumn {
                        selectables: vec![SelectableStackElement::Column("id".to_string())],
                        column_name: "id".to_string(),
                    },
                    SelectableColumn {
                        selectables: vec![SelectableStackElement::Column("name".to_string())],
                        column_name: "name".to_string(),
                    },
                    SelectableColumn {
                        selectables: vec![SelectableStackElement::Column("age".to_string())],
                        column_name: "age".to_string(),
                    },
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
        assert_eq!(
            statement,
            SelectStatement {
                table_name: "guests".to_string(),
                table_aliases: TableAliases(HashMap::new()),
                mode: SelectMode::Distinct,
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                }],
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }
        );
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
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "money"),
            token(TokenTypes::GreaterEquals, ">="),
            token(TokenTypes::RealLiteral, "300.0"),
            token(TokenTypes::Or, "OR"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "20"),
            token(TokenTypes::And, "AND"),
            token(TokenTypes::Identifier, "money"),
            token(TokenTypes::GreaterEquals, ">="),
            token(TokenTypes::RealLiteral, "100.5"),
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

        let expected = SelectStatement {
            table_name: "people".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::Distinct,
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Add),
                    ],
                    column_name: "age + money".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Integer(2)),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Modulo),
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Divide),
                        SelectableStackElement::MathOperator(MathOperator::Subtract),
                        SelectableStackElement::MathOperator(MathOperator::Subtract),
                        SelectableStackElement::MathOperator(MathOperator::Multiply),
                    ],
                    column_name: "2 * ( ( age - ( id % age - id / money ) ) )".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Value(Value::Real(300.0)),
                        SelectableStackElement::Operator(Operator::GreaterEquals),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::Value(Value::Integer(20)),
                        SelectableStackElement::Operator(Operator::GreaterThan),
                        SelectableStackElement::LogicalOperator(LogicalOperator::Not),
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Value(Value::Real(100.5)),
                        SelectableStackElement::Operator(Operator::GreaterEquals),
                        SelectableStackElement::LogicalOperator(LogicalOperator::And),
                        SelectableStackElement::LogicalOperator(LogicalOperator::Or),
                    ],
                    column_name: "money >= 300.0 OR NOT age > 20 AND money >= 100.5".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Multiply),
                    ],
                    column_name: "id * age".to_string(),
                }],
                directions: vec![OrderByDirection::Asc],
            }),
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_limit_but_no_offset_is_generated_correctly() {
        // SELECT id, name FROM users LIMIT 5;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                },
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("name".to_string())],
                    column_name: "name".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: 5,
                offset: None,
            }),
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_where_clause_only_is_generated_correctly() {
        // SELECT name FROM users WHERE name = 'John';
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::StringLiteral, "John"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::Column("name".to_string())],
                column_name: "name".to_string(),
            }],
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("name".to_string()),
                    SelectableStackElement::Value(Value::Text("John".to_string())),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "name = 'John'".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_distinct_and_multiple_columns_is_generated_correctly() {
        // SELECT DISTINCT name, age FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Distinct, "DISTINCT"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "age"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::Distinct,
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("name".to_string())],
                    column_name: "name".to_string(),
                },
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("age".to_string())],
                    column_name: "age".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_order_by_only_is_generated_correctly() {
        // SELECT id, name FROM users ORDER BY name DESC;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Desc, "DESC"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                },
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("name".to_string())],
                    column_name: "name".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("name".to_string())],
                    column_name: "name".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: None,
        };
        assert_eq!(expected, statement);
    }
}
