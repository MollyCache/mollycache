use std::collections::HashSet;

use crate::db::table::core::table::Table;
use crate::db::table::operations::helpers::common::get_row_indicies_matching_clauses;
use crate::interpreter::ast::DeleteStatement;

pub fn delete(
    table: &mut Table,
    statement: DeleteStatement,
    is_transaction: bool,
) -> Result<Vec<usize>, String> {
    let mut row_indicies_to_delete = get_row_indicies_matching_clauses(
        table,
        &statement.where_clause,
        &statement.order_by_clause,
        &statement.limit_clause,
    )?;
    // We get omega saved here by the fact that we don't need to guarentee the order of the rows after rollbacks.
    // This means we can swap the semi-deleted rows to the end of the table and then set the length of the table
    // to the length of the table minus the number of semi-deleted rows. Then on rollback we can just extend the length of the table.
    // to then include the deleted rows. if we commit, we pop off the end of the table until at the desired length.
    swap_remove_bulk(table, &mut row_indicies_to_delete, is_transaction)?;
    Ok(row_indicies_to_delete)
}

fn swap_remove_bulk(
    table: &mut Table,
    row_indicies: &mut Vec<usize>,
    is_transaction: bool,
) -> Result<(), String> {
    if table.len() == 0 {
        if row_indicies.len() != 0 {
            unreachable!();
        }
        return Ok(());
    }
    let table_len = table.len() - 1;
    let mut row_indicies_set = row_indicies.iter().collect::<HashSet<&usize>>();
    let mut right_pointer = 0;
    let mut iter = row_indicies.iter().rev(); // We recieve the indexes in ascending order,
    // We reverse them to get rid of the furtherst indexes first.

    while let Some(to_swap) = iter.next() {
        if *to_swap == (table_len - right_pointer) {
            row_indicies_set.remove(to_swap);
            right_pointer += 1;
        } else {
            table.swap(*to_swap, table_len - right_pointer);
            row_indicies_set.remove(to_swap);
            right_pointer += 1;
        }
    }
    if is_transaction {
        table.set_length(table.len() - right_pointer);
    } else {
        for _ in 0..right_pointer {
            table.pop();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::{row::Row, value::Value};
    use crate::db::table::test_utils::{assert_table_rows_eq_unordered, default_table};
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::{
        Operator, OrderByClause, OrderByDirection, SelectableColumn, SelectableStackElement, TableAliases
    };
    use std::collections::HashMap;

    #[test]
    fn delete_from_table_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(2)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "id = 2".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![
                Value::Integer(1),
                Value::Text("John".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
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
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn delete_from_table_with_all_clauses_works_correctly() {
        let mut table = default_table();
        table.set_rows(vec![
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
            Row(vec![
                Value::Integer(5),
                Value::Text("John".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
            ]),
            Row(vec![
                Value::Integer(6),
                Value::Text("John".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(7),
                Value::Text("John".to_string()),
                Value::Integer(35),
                Value::Real(3000.0),
            ]),
        ]);
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("name".to_string()),
                    SelectableStackElement::Value(Value::Text("John".to_string())),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "name = 'John'".to_string(),
            }),
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: Some(LimitClause {
                limit: 1,
                offset: Some(2),
            }),
        };
        let result = delete(&mut table, statement, false);
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
            Row(vec![
                Value::Integer(6),
                Value::Text("John".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(7),
                Value::Text("John".to_string()),
                Value::Integer(35),
                Value::Real(3000.0),
            ]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn delete_multiple_rows_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(1)),
                    SelectableStackElement::Operator(Operator::GreaterThan),
                ],
                column_name: "id = 2".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
        let row_indicies = result.unwrap();
        assert_eq!(vec![1, 2, 3], row_indicies);
        let expected = vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
            Value::Integer(25),
            Value::Real(1000.0),
        ])];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn delete_all_rows_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
        let expected = vec![];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn delete_from_empty_table_works_correctly() {
        let mut table = default_table();
        table.set_rows(vec![]);
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
    }

    #[test]
    fn delete_with_complex_where_and_limit_edge_case() {
        let mut table = default_table();
        table.set_rows(vec![
            Row(vec![
                Value::Integer(10),
                Value::Text("Alice".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
            ]),
            Row(vec![
                Value::Integer(20),
                Value::Text("Bob".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(30),
                Value::Text("Charlie".to_string()),
                Value::Integer(35),
                Value::Real(3000.0),
            ]),
            Row(vec![
                Value::Integer(40),
                Value::Text("David".to_string()),
                Value::Integer(40),
                Value::Real(4000.0),
            ]),
            Row(vec![
                Value::Integer(50),
                Value::Text("Eve".to_string()),
                Value::Integer(45),
                Value::Real(5000.0),
            ]),
        ]);
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("age".to_string()),
                    SelectableStackElement::Value(Value::Integer(30)),
                    SelectableStackElement::Operator(Operator::GreaterEquals),
                ],
                column_name: "id = 2".to_string(),
            }),
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                }],
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: Some(LimitClause {
                limit: 2,
                offset: Some(1),
            }),
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
        let deleted_indices = result.unwrap();
        assert_eq!(deleted_indices.len(), 2);
        let expected = vec![
            Row(vec![
                Value::Integer(10),
                Value::Text("Alice".to_string()),
                Value::Integer(25),
                Value::Real(1000.0),
            ]),
            Row(vec![
                Value::Integer(20),
                Value::Text("Bob".to_string()),
                Value::Integer(30),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(50),
                Value::Text("Eve".to_string()),
                Value::Integer(45),
                Value::Real(5000.0),
            ]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn delete_single_row_from_single_row_table_returns_correct_index() {
        let mut table = default_table();
        table.set_rows(vec![Row(vec![
            Value::Integer(42),
            Value::Text("OnlyOne".to_string()),
            Value::Integer(99),
            Value::Real(123.45),
        ])]);
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            table_aliases: TableAliases(HashMap::new()),
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(42)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "id = 42".to_string(),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement, false);
        assert!(result.is_ok());
        let deleted_indices = result.unwrap();
        assert_eq!(deleted_indices, vec![0]);
        assert_eq!(table.get_rows_clone().len(), 0);
    }
}
