pub mod where_clause;
use crate::db::table::{Table, Value};
use crate::cli::ast::SelectStatement;
use crate::cli::ast::SelectStatementColumns;
use crate::db::table::common::validate_and_clone_row;


pub fn select(table: &Table, statement: SelectStatement) -> Result<Vec<Vec<Value>>, String> {
    let mut rows: Vec<Vec<Value>> = vec![];
    if let Some(where_clause) = statement.where_clause {
        for row in table.rows.iter() {
            if where_clause::matches_where_clause(table, &row, &where_clause) {
                rows.push(get_columns_from_row(table, &row, &statement.columns)?);
            }
        }
    } else {
        for row in table.rows.iter() {
            rows.push(get_columns_from_row(table, &row, &statement.columns)?);
        }
    }
    return Ok(rows);
}

pub fn get_columns_from_row(table: &Table, row: &Vec<Value>, selected_columns: &SelectStatementColumns) -> Result<Vec<Value>, String> {
    let mut row_values: Vec<Value> = vec![];
    if *selected_columns == SelectStatementColumns::All {
        return Ok(validate_and_clone_row(table, row)?);
    } else {
        let specific_selected_columns = selected_columns.columns()?;
        for (i, column) in table.columns.iter().enumerate() {
            if (*specific_selected_columns).contains(&column.name) {
                row_values.push(row[i].clone());
            }
        }
    }
    return Ok(row_values);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition};
    use crate::cli::ast::SelectStatementColumns;
    use crate::cli::ast::Operator;
    use crate::cli::ast::WhereClause;

    fn default_table() -> Table {
        Table {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {name: "id".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "name".to_string(), data_type: DataType::Text, constraints: vec![]},
                ColumnDefinition {name: "age".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "money".to_string(), data_type: DataType::Real, constraints: vec![]},
            ],
            rows: vec![
                vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
                vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
                vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
                vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
            ],
        }
    }

    #[test]
    fn select_with_all_tokens_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_specific_columns_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::Specific(vec!["name".to_string(), "age".to_string()]),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Text("John".to_string()), Value::Integer(25)],
            vec![Value::Text("Jane".to_string()), Value::Integer(30)],
            vec![Value::Text("Jim".to_string()), Value::Integer(35)],
            vec![Value::Null, Value::Integer(40)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: Some(WhereClause {
                column: "name".to_string(),
                operator: Operator::Equals,
                value: Value::Text("John".to_string()),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_selected_columns() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::Specific(vec!["name".to_string(), "age".to_string()]),
            where_clause: Some(WhereClause {
                column: "money".to_string(),
                operator: Operator::Equals,
                value: Value::Real(1000.0),
            }),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Text("John".to_string()), Value::Integer(25)],
        ];
        assert_eq!(expected, result.unwrap());
    }
}