use crate::db::table::core::{row::Row, table::Table, value::Value};
use crate::db::table::operations::helpers::common::{get_column, get_columns};
use crate::db::table::operations::helpers::order_by_clause::apply_order_by_from_precomputed;
use crate::interpreter::ast::{SelectMode, SelectStatement};
use std::collections::{HashMap, HashSet};

pub fn select_statement(table: &Table, statement: &SelectStatement) -> Result<Vec<Row>, String> {
    let mut rows = vec![];
    let (limit, offset) = statement.limit_clause.as_ref().map_or((-1, 0), |stmt| {
        (stmt.limit as i64, stmt.offset.map_or(0, |val| val))
    });

    let mut order_by_columns_precomputed = vec![];

    let mut distinct_map = match statement.mode {
        SelectMode::All => None,
        SelectMode::Distinct => Some(HashSet::new()),
    };

    let alias_to_computed_index = statement
        .columns
        .iter()
        .enumerate()
        .map(|(i, column)| (column.column_name.clone(), i))
        .collect::<HashMap<String, usize>>();

    for row in table.iter().skip(if statement.order_by_clause.is_none() {
        offset
    } else {
        0
    }) {
        if limit != -1 && rows.len() as i64 >= limit && statement.order_by_clause.is_none() {
            break;
        }
        let columns = get_columns(table, row, &statement.columns, None, None)?;
        if let Some(stmt) = &statement.where_clause {
            if let Value::Integer(val) = get_column(
                table,
                row,
                stmt,
                Some(&columns),
                Some(&alias_to_computed_index),
            )? {
                if val == 0 {
                    continue;
                }
            } else {
                return Err("WHERE condition did not return a boolean".to_string());
            }
        }

        if let Some(map) = &mut distinct_map {
            if map.insert(columns.clone()) {
                if let Some(stmt) = &statement.order_by_clause {
                    order_by_columns_precomputed.push(get_columns(
                        table,
                        row,
                        &stmt.columns,
                        Some(&columns),
                        Some(&alias_to_computed_index),
                    )?);
                }
                rows.push(columns);
            }
        } else {
            if let Some(stmt) = &statement.order_by_clause {
                order_by_columns_precomputed.push(get_columns(
                    table,
                    row,
                    &stmt.columns,
                    Some(&columns),
                    Some(&alias_to_computed_index),
                )?);
            }
            rows.push(columns);
        }
    }

    if let Some(stmt) = &statement.order_by_clause {
        apply_order_by_from_precomputed(&mut rows, order_by_columns_precomputed, Row(vec![]), stmt);
        if limit != -1 || offset != 0 {
            // If offset exceeds the result size, return empty set (SQLite-compatible behavior)
            if offset >= rows.len() {
                rows = vec![];
            } else {
                let end = if (limit == -1) || (offset + limit as usize > rows.len()) {
                    rows.len()
                } else {
                    offset + limit as usize
                };
                rows = rows[offset..end].to_vec();
            }
        }
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::column::ColumnDefinition;
    use crate::db::table::core::value::DataType;
    use crate::db::table::core::{row::Row, value::Value};
    use crate::db::table::test_utils::{
        assert_table_rows_eq, assert_table_rows_eq_unordered, default_table,
    };
    use crate::interpreter::ast::SelectMode;
    use crate::interpreter::ast::{
        LimitClause, MathOperator, Operator, OrderByClause, OrderByDirection, SelectableColumn,
        SelectableStackElement, TableAliases,
    };
    use std::collections::HashMap;

    #[test]
    fn select_with_all_tokens_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
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
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![
                Value::Integer(1),
                Value::Text("John".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
            ]),
            Row(vec![
                Value::Integer(2),
                Value::Text("Jane".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(3),
                Value::Text("Jim".to_string()),
                Value::Integer(35),
                Value::Real(3000.0),
            ]),
            Row(vec![
                Value::Integer(4),
                Value::Null,
                Value::Integer(40),
                Value::Real(4000.0),
            ]),
        ];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_specific_columns_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
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
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![Value::Text("John".to_string()), Value::Integer(25)]),
            Row(vec![Value::Text("Jane".to_string()), Value::Integer(30)]),
            Row(vec![Value::Text("Jim".to_string()), Value::Integer(35)]),
            Row(vec![Value::Null, Value::Integer(40)]),
        ];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::All],
                column_name: "*".to_string(),
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
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
            Value::Integer(25),
            Value::Real(1000.0),
        ])];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_selected_columns() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
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
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("money".to_string()),
                    SelectableStackElement::Value(Value::Real(1000.0)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "money = 1000.0".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![Row(vec![
            Value::Text("John".to_string()),
            Value::Integer(25),
        ])];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_limit_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::All],
                column_name: "*".to_string(),
            }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: 1,
                offset: Some(1),
            }),
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![Row(vec![
            Value::Integer(2),
            Value::Text("Jane".to_string()),
            Value::Integer(30),
            Value::Real(2000.0),
        ])];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_table_returns_error() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::All],
                column_name: "*".to_string(),
            }],
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("column_not_included".to_string()),
                    SelectableStackElement::Value(Value::Text("John".to_string())),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "column_not_included = 'John'".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid column name: column_not_included"
        );
    }

    #[test]
    fn select_with_order_by_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::All],
                column_name: "*".to_string(),
            }],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("money".to_string())],
                    column_name: "money".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![
                Value::Integer(4),
                Value::Null,
                Value::Integer(40),
                Value::Real(4000.0),
            ]),
            Row(vec![
                Value::Integer(3),
                Value::Text("Jim".to_string()),
                Value::Integer(35),
                Value::Real(3000.0),
            ]),
            Row(vec![
                Value::Integer(2),
                Value::Text("Jane".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(1),
                Value::Text("John".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
            ]),
        ];
        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_distinct_mode_is_generated_correctly() {
        let mut table = Table::new(
            "users".to_string(),
            vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![],
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Text,
                    constraints: vec![],
                },
            ],
        );
        table.set_rows(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(3), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(4), Value::Null]),
        ]);
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::Distinct,
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::Column("name".to_string())],
                column_name: "name".to_string(),
            }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![Value::Text("John".to_string())]),
            Row(vec![Value::Text("Jane".to_string())]),
            Row(vec![Value::Null]),
        ];
        assert_table_rows_eq_unordered(expected, result.unwrap());
    }

    #[test]
    fn select_with_math_and_logic_operations_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
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
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Divide),
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Add),
                    ],
                    column_name: "money / age + id".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Value(Value::Integer(2000)),
                        SelectableStackElement::Operator(Operator::GreaterEquals),
                    ],
                    column_name: "money >= 2000".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Integer(1)),
                        SelectableStackElement::Value(Value::Real(1.0)),
                        SelectableStackElement::MathOperator(MathOperator::Add),
                    ],
                    column_name: "1 + 1.0".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Divide),
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Add),
                    ],
                    column_name: "money / age + id".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: None,
        };

        let result = select_statement(&table, &statement);
        assert!(result.is_ok());

        let expected = vec![
            Row(vec![
                Value::Integer(4),
                Value::Real(4040.0),
                Value::Real(104.0),
                Value::Integer(1),
                Value::Real(2.0),
            ]),
            Row(vec![
                Value::Integer(3),
                Value::Real(3035.0),
                Value::Real(88.71428571428571),
                Value::Integer(1),
                Value::Real(2.0),
            ]),
            Row(vec![
                Value::Integer(2),
                Value::Real(2030.0),
                Value::Real(68.66666666666667),
                Value::Integer(1),
                Value::Real(2.0),
            ]),
            Row(vec![
                Value::Integer(1),
                Value::Real(1025.0),
                Value::Real(41.0),
                Value::Integer(0),
                Value::Real(2.0),
            ]),
        ];

        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_operators_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Integer(1)),
                        SelectableStackElement::Value(Value::Real(1.0)),
                        SelectableStackElement::Operator(Operator::GreaterEquals),
                    ],
                    column_name: "1 >= -1.0".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Real(3000.0)),
                        SelectableStackElement::Value(Value::Real(2000.0)),
                        SelectableStackElement::Operator(Operator::LessThan),
                    ],
                    column_name: "3000.0 < 2000.0".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Integer(10)),
                        SelectableStackElement::Value(Value::Integer(-11)),
                        SelectableStackElement::Operator(Operator::LessEquals),
                    ],
                    column_name: "10 <= -11".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Value(Value::Integer(10)),
                        SelectableStackElement::Value(Value::Real(10.0)),
                        SelectableStackElement::Operator(Operator::Equals),
                    ],
                    column_name: "10 == 10.0".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };

        let result = select_statement(&table, &statement);
        assert!(result.is_ok());

        let expected = vec![
            Row(vec![
                Value::Integer(1),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(1),
            ]),
            Row(vec![
                Value::Integer(1),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(1),
            ]),
            Row(vec![
                Value::Integer(1),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(1),
            ]),
            Row(vec![
                Value::Integer(1),
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(1),
            ]),
        ];

        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_aliases_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("money".to_string()),
                    SelectableStackElement::Column("age".to_string()),
                    SelectableStackElement::MathOperator(MathOperator::Divide),
                ],
                column_name: "some_alias".to_string(),
            }],
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("some_alias".to_string()),
                    SelectableStackElement::Value(Value::Integer(80)),
                    SelectableStackElement::Operator(Operator::GreaterThan),
                ],
                column_name: "some_alias > 80".to_string(),
            }),
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("some_alias".to_string())],
                    column_name: "some_alias".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: None,
        };

        let result = select_statement(&table, &statement);
        assert!(result.is_ok());

        let expected = vec![
            Row(vec![Value::Real(100.0)]),
            Row(vec![Value::Real(85.71428571428571)]),
        ];

        assert_table_rows_eq(expected, result.unwrap());
    }

    #[test]
    fn select_with_nonexisting_aliases_crashes() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("money".to_string()),
                    SelectableStackElement::Column("age".to_string()),
                    SelectableStackElement::MathOperator(MathOperator::Divide),
                ],
                column_name: "some_alias".to_string(),
            }],
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("nonexisting_alias".to_string()),
                    SelectableStackElement::Value(Value::Integer(80)),
                    SelectableStackElement::Operator(Operator::GreaterThan),
                ],
                column_name: "nonexisting_alias > 80".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };

        let result = select_statement(&table, &statement);
        assert!(result.is_err());
        assert!(result.err().unwrap() == "Invalid column name: nonexisting_alias");
    }

    #[test]
    fn select_using_alias_in_selected_columns_crashes() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            mode: SelectMode::All,
            columns: vec![
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("money".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Divide),
                    ],
                    column_name: "some_alias".to_string(),
                },
                SelectableColumn {
                    selectables: vec![
                        SelectableStackElement::Column("some_alias".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                        SelectableStackElement::MathOperator(MathOperator::Multiply),
                    ],
                    column_name: "some_alias * age".to_string(),
                },
            ],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };

        let result = select_statement(&table, &statement);
        assert!(result.is_err());
        assert!(result.err().unwrap() == "Invalid column name: some_alias");
    }
}
