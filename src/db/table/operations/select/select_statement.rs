use crate::db::table::core::{row::Row, table::Table};
use crate::db::table::operations::helpers::common::get_columns_from_row;
use crate::db::table::operations::helpers::order_by_clause::apply_order_by_from_precomputed;
use crate::db::table::operations::helpers::where_clause::row_matches_where_stack;
use crate::interpreter::ast::{SelectMode, SelectStatement};
use std::collections::HashSet;

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

    for row in table.iter().skip(if statement.order_by_clause.is_none() {
        offset
    } else {
        0
    }) {
        if limit != -1 && rows.len() as i64 >= limit && statement.order_by_clause.is_none() {
            break;
        } else if statement.where_clause.as_ref().map_or_else(
            || Ok(true),
            |stmt| row_matches_where_stack(table, row, &stmt),
        )? {
            let columns = get_columns_from_row(table, row, &statement.columns)?;
            if let Some(map) = &mut distinct_map {
                if map.insert(columns.clone()) {
                    rows.push(columns);
                    if let Some(stmt) = &statement.order_by_clause {
                        order_by_columns_precomputed.push(get_columns_from_row(
                            table,
                            row,
                            &stmt.columns,
                        )?);
                    }
                }
            } else {
                rows.push(columns);
                if let Some(stmt) = &statement.order_by_clause {
                    order_by_columns_precomputed.push(get_columns_from_row(
                        table,
                        row,
                        &stmt.columns,
                    )?);
                }
            }
        }
    }

    if let Some(stmt) = &statement.order_by_clause {
        apply_order_by_from_precomputed(&mut rows, order_by_columns_precomputed, Row(vec![]), stmt);
        if limit != -1 || offset != 0 {
            let end = if (limit == -1) || (offset + limit as usize > rows.len()) {
                rows.len()
            } else {
                offset + limit as usize
            };
            rows = rows[offset..end].to_vec();
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
    use crate::db::table::test_utils::{assert_table_rows_eq_unordered, default_table};
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::SelectMode;
    use crate::interpreter::ast::SelectStatementColumn;
    use crate::interpreter::ast::SelectStatementTable;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::{
        LimitClause, Operator, OrderByClause, OrderByDirection, SelectableStack,
        SelectableStackElement,
    };

    #[test]
    fn select_with_all_tokens_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec![SelectStatementColumn::new("*".to_string())],
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
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_specific_columns_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(SelectStatementColumn::new("name".to_string())),
                    SelectableStackElement::Column(SelectStatementColumn::new("age".to_string())),
                ],
            },
            column_names: vec![
                SelectStatementColumn::new("name".to_string()),
                SelectStatementColumn::new("age".to_string()),
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
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec![SelectStatementColumn::new("*".to_string())],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("name".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Text("John".to_string())),
            })]),
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
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_selected_columns() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column(SelectStatementColumn::new("name".to_string())),
                    SelectableStackElement::Column(SelectStatementColumn::new("age".to_string())),
                ],
            },
            column_names: vec![
                SelectStatementColumn::new("name".to_string()),
                SelectStatementColumn::new("age".to_string()),
            ],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("money".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Real(1000.0)),
            })]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_ok());
        let expected = vec![Row(vec![
            Value::Text("John".to_string()),
            Value::Integer(25),
        ])];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_limit_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec![SelectStatementColumn::new("*".to_string())],
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
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_table_returns_error() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec![SelectStatementColumn::new("*".to_string())],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("column_not_included".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Text("John".to_string())),
            })]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Column column_not_included does not exist in table users"
        );
    }

    #[test]
    fn select_with_order_by_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: SelectStatementTable::new("users".to_string()),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec![SelectStatementColumn::new("*".to_string())],
            where_clause: None,
            order_by_clause: Some(OrderByClause {
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::Column(SelectStatementColumn::new(
                        "money".to_string(),
                    ))],
                },
                column_names: vec![SelectStatementColumn::new("money".to_string())],
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
        assert_eq!(expected, result.unwrap());
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
            table_name: SelectStatementTable::new("users".to_string()),
            column_names: vec![SelectStatementColumn::new("name".to_string())],
            mode: SelectMode::Distinct,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column(SelectStatementColumn::new(
                    "name".to_string(),
                ))],
            },
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
}
