use std::collections::{HashMap, VecDeque};

use crate::db::table::{Table, Value, Row};
use crate::interpreter::ast::InsertIntoStatement;
use crate::db::table::helpers::common::validate_and_clone_row;


pub fn insert(table: &mut Table, statement: InsertIntoStatement) -> Result<Vec<usize>, String> {
    // Validate columns
    if let Some(columns) = &statement.columns {
        for column in columns {
            if table.columns.iter().find(|c| c.name == *column).is_none() {
                return Err(format!("Column '{}' does not exist in table", column));
            }
        }
    }
    
    let mut rows: Vec<Row> = vec![];
    // Creates a hash map from the statement values with the columns as the keys
    // The values are stored in a queue to match the order of the columns, we push back to the queue
    // and then pop off the front when creating the rows.
    // Todo: make this logic simpler.
    if let Some(statement_columns) = &statement.columns {
        let mut map: HashMap<&String, VecDeque<Value>> = HashMap::new();
        for (i, column) in statement_columns.iter().enumerate() {
            map.insert(column, VecDeque::new());
            for row in statement.values.iter() {
                map.get_mut(column).unwrap().push_back(row[i].clone());
            }
        }
        for _ in 0..statement.values.len() {
            let mut row: Row = Row(vec![]);
            for table_column in table.columns.iter() {
                if map.contains_key(&table_column.name) {
                    let queue = map.get_mut(&table_column.name).unwrap();
                    let value = queue.pop_front().unwrap();
                    row.push(value);
                }
                else {
                    row.push(Value::Null);
                }
            }
            rows.push(row);
        }
    } else {
        // Inserts entire row in the order provided in the statement
        for row in statement.values {
            let row_values = validate_and_clone_row(table, &Row(row))?;
            rows.push(row_values);
        }
    }

    // Insert rows
    let mut row_indicies: Vec<usize> = vec![];
    for row in rows {
        table.push(row);
        row_indicies.push(table.rows.len() - 1);
    }
    return Ok(row_indicies);
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition};

    fn default_table() -> Table {
        Table::new(
            "users".to_string(), 
            vec![
                ColumnDefinition {name: "id".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "name".to_string(), data_type: DataType::Text, constraints: vec![]},
                ColumnDefinition {name: "age".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "money".to_string(), data_type: DataType::Real, constraints: vec![]},
            ]
        )
    }

    #[test]
    fn insert_into_table_is_generated_correctly() {
        let mut table = default_table();
        let statement = InsertIntoStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]],
        };
        assert!(insert(&mut table, statement).is_ok());
        let expected = vec![Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)])];
        assert_eq!(table.get_rows_clone(), expected);
    }

    #[test]
    fn insert_into_table_with_columns_is_generated_correctly() {
        let mut table = default_table();
        table.set_rows(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
        ]);
        let statement = InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec!["id".to_string(), "name".to_string()]),
            values: vec![
                vec![Value::Integer(3), Value::Text("John".to_string()),],
                vec![Value::Integer(4), Value::Text("Jane".to_string())],
            ],
        };
        let result = insert(&mut table, statement);
        assert!(result.is_ok());
        let row_indicies = result.unwrap();
        assert_eq!(row_indicies, vec![2, 3]);
        let expected = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("John".to_string()), Value::Null, Value::Null]),
            Row(vec![Value::Integer(4), Value::Text("Jane".to_string()), Value::Null, Value::Null]),
        ];
        assert_eq!(expected, table.get_rows_clone());
    }
}