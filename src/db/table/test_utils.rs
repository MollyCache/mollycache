#[cfg(test)]
use crate::db::table::{Table, Value, DataType, ColumnDefinition};
#[cfg(test)]
use crate::cli::ast::OrderByDirection;
#[cfg(test)]
use std::cmp::Ordering;


#[cfg(test)]
pub fn default_table() -> Table {
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

#[cfg(test)]
pub fn assert_table_rows_eq_unordered(mut expected: Vec<Vec<Value>>, mut actual: Vec<Vec<Value>>) {
    expected.sort_by(|a, b| {
        let mut i = 0;
        while i < a.len() && i < b.len() && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal {
            i += 1;
        }
        a[i].compare(&b[i], &OrderByDirection::Asc)
    });
    actual.sort_by(|a, b| {
        let mut i = 0;
        while i < a.len() && i < b.len() && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal {
            i += 1;
        }
        a[i].compare(&b[i], &OrderByDirection::Asc)
    });
    assert_eq!(expected, actual);
}