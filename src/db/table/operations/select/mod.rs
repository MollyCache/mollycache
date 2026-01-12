pub mod select_statement;
pub mod set_operator_evaluator;
use crate::db::table::operations::helpers::order_by_clause::apply_order_by_from_precomputed;
use crate::db::{
    database::Database,
    table::core::{row::Row, table::Table},
};
use crate::interpreter::ast::{
    SelectStatementStack, SelectStatementStackElement, SelectableColumn, SetOperator,
};

pub fn select_statement_stack(
    database: &Database,
    statement: SelectStatementStack,
) -> Result<Vec<Row>, String> {
    let mut evaluator = set_operator_evaluator::SetOperatorEvaluator::new();
    let mut column_names: Option<Vec<String>> = None;

    // TODO: so ugly and also just false. Needed in some sort of way for now. See later TODO about dealing with 2+ tables
    let mut has_first_table = false;
    let mut temp_tables = Vec::new();

    for element in statement.elements {
        match element {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                let table = if select_statement.table_name.is_empty() {
                    let mut t = Table::new("".to_string(), vec![]);
                    t.set_rows(vec![Row(vec![])]);
                    temp_tables.push(t);
                    temp_tables.last().unwrap()
                } else {
                    database.get_table_with_aliases(
                        &select_statement.table_name,
                        &select_statement.table_aliases,
                    )?
                };

                let expanded_column_names =
                    expand_all_column_names(table, &select_statement.columns)?;
                match &column_names {
                    Some(column_names) => {
                        if expanded_column_names.len() != column_names.len() {
                            return Err(format!(
                                "Columns mismatch between SELECT statements in Union"
                            ));
                        } else if expanded_column_names
                            .iter()
                            .zip(column_names)
                            .filter(|&(a, b)| a != b)
                            .count()
                            != 0
                        {
                            return Err(format!(
                                "Columns mismatch between SELECT statements in Union"
                            ));
                        }
                    }
                    None => {
                        column_names = Some(expanded_column_names);
                    }
                }

                let rows = select_statement::select_statement(table, &select_statement)?;
                evaluator.push(rows);

                has_first_table = true;
            }
            SelectStatementStackElement::SetOperator(set_operator) => match set_operator {
                SetOperator::UnionAll => {
                    evaluator.union_all()?;
                }
                SetOperator::Union => {
                    evaluator.union()?;
                }
                SetOperator::Intersect => {
                    evaluator.intersect()?;
                }
                SetOperator::Except => {
                    evaluator.except()?;
                }
            },
        }
    }
    let mut result = evaluator.result()?;
    if let Some(order_by_clause) = statement.order_by_clause {
        if has_first_table {
            // TODO: this is just plain false when working with 2+ tables
            // When using ORDER BY at the end of set operations on SELECTs, the ordering columns are guaranteed (?) to be present in the selected columns
            // TODO: this ^ is not quite accurate
            let mut result_indices = vec![];
            for order_by_column in &order_by_clause.columns {
                result_indices.push(
                    column_names
                        .as_ref()
                        .ok_or_else(|| "No column names found".to_string())?
                        .iter()
                        .position(|column_name| column_name == &order_by_column.column_name)
                        .ok_or_else(|| {
                            "Ordering column name not found in selected columns".to_string()
                        })?,
                );
            }

            let precomputed = result
                .iter()
                .map(|row| {
                    let mut order_columns = vec![];
                    result_indices.iter().for_each(|i| {
                        order_columns.push(row[*i].clone());
                    });
                    Row(order_columns)
                })
                .collect::<Vec<Row>>();
            apply_order_by_from_precomputed(
                &mut result,
                precomputed,
                Row(vec![]),
                &order_by_clause,
            );
        } else {
            unreachable!();
        }
    }

    // TODO: if LIMIT without ORDER BY, apply LIMIT at the beginning / after the WHERE
    if let Some(limit_clause) = statement.limit_clause {
        let offset = limit_clause.offset.unwrap_or(0);
        // If offset exceeds the result size, return empty set (SQLite-compatible behavior)
        if offset >= result.len() {
            result = vec![];
        } else {
            let end = if limit_clause.limit + offset > result.len() {
                result.len()
            } else {
                limit_clause.limit + offset
            };
            result = result[offset..end].to_vec();
        }
    }
    Ok(result)
}

// TODO: add this logic in evaluation too
fn expand_all_column_names(
    table: &Table,
    columns: &Vec<SelectableColumn>,
) -> Result<Vec<String>, String> {
    let mut new = vec![];
    for column in columns {
        if *column.column_name == "*".to_string() {
            for name in table.get_column_names()? {
                if !new.contains(name) {
                    new.push(name.clone());
                }
            }
        } else {
            new.push(column.column_name.clone());
        }
    }
    Ok(new)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::Value;
    use crate::db::table::test_utils::{assert_table_rows_eq, default_database};
    use crate::interpreter::ast::{
        LogicalOperator, Operator, SelectMode, SelectStatement, SelectableColumn,
        SelectableStackElement, TableAliases,
    };
    use std::collections::HashMap;

    #[test]
    fn select_statement_stack_with_multiple_set_operators_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(
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
                },
            )],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
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
    fn select_statement_stack_with_set_operator_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    table_aliases: TableAliases(HashMap::new()),
                    mode: SelectMode::All,
                    columns: vec![SelectableColumn {
                        selectables: vec![SelectableStackElement::All],
                        column_name: "*".to_string(),
                    }],
                    where_clause: Some(SelectableColumn {
                        selectables: vec![
                            SelectableStackElement::Column("id".to_string()),
                            SelectableStackElement::Value(Value::Integer(1)),
                            SelectableStackElement::Operator(Operator::Equals),
                        ],
                        column_name: "id = 1".to_string(),
                    }),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SelectStatement(SelectStatement {
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
                }),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            ],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
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
    fn select_statement_stack_works_correctly_with_multiple_set_operators() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
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
                }),
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    table_aliases: TableAliases(HashMap::new()),
                    mode: SelectMode::All,
                    columns: vec![SelectableColumn {
                        selectables: vec![SelectableStackElement::All],
                        column_name: "*".to_string(),
                    }],
                    where_clause: Some(SelectableColumn {
                        selectables: vec![
                            SelectableStackElement::Column("id".to_string()),
                            SelectableStackElement::Value(Value::Integer(2)),
                            SelectableStackElement::Operator(Operator::Equals),
                            SelectableStackElement::Column("id".to_string()),
                            SelectableStackElement::Value(Value::Integer(2)),
                            SelectableStackElement::Operator(Operator::Equals),
                            SelectableStackElement::LogicalOperator(LogicalOperator::Or),
                        ],
                        column_name: "id = 1 OR id = 2".to_string(),
                    }),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    table_aliases: TableAliases(HashMap::new()),
                    mode: SelectMode::All,
                    columns: vec![SelectableColumn {
                        selectables: vec![SelectableStackElement::All],
                        column_name: "*".to_string(),
                    }],
                    where_clause: Some(SelectableColumn {
                        selectables: vec![
                            SelectableStackElement::Column("id".to_string()),
                            SelectableStackElement::Value(Value::Integer(1)),
                            SelectableStackElement::Operator(Operator::Equals),
                        ],
                        column_name: "id = 1".to_string(),
                    }),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SetOperator(SetOperator::Except),
            ],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
        assert!(result.is_ok());
        let expected = vec![Row(vec![
            Value::Integer(2),
            Value::Text("Jane".to_string()),
            Value::Integer(30),
            Value::Real(2000.0),
        ])];
        assert_table_rows_eq(expected, result.unwrap());
    }
}
