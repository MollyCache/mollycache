use std::cmp::Ordering;

use crate::interpreter::ast::OrderByClause;
use crate::db::table::{Row};

// TODO: got changed
pub fn perform_comparisons(row1: &Row, row2: &Row, order_by_clauses: &Vec<OrderByClause>) -> Ordering {
    /*
    let mut result = Ordering::Equal;
    for comparison in order_by_clauses {
        let index = get_index_of_column(columns, &comparison.column);
        let index = match index {
            Ok(index) => index,
            Err(_) => unreachable!(),
        };
        let ordering = row1[index].compare(&row2[index], &comparison.direction);
        if ordering != Ordering::Equal {
            result = ordering;
            break;
        }
    }
    return result;
    */
    return Ordering::Equal;
}

fn get_index_of_column(columns: &Vec<&String>, column_name: &String) -> Result<usize, String> {
    let result = columns.iter().position(|column| column_name == *column);
    if let Some(index) = result {
        return Ok(index);
    }
    else {
        return Err(format!("Column {} does not exist in table", column_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition, Row, RowStack};
    use crate::interpreter::ast::OrderByDirection;

    fn default_table() -> Table {
        Table {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {name: "id".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "name".to_string(), data_type: DataType::Text, constraints: vec![]},
                ColumnDefinition {name: "money".to_string(), data_type: DataType::Real, constraints: vec![]},
                ColumnDefinition {name: "some_data".to_string(), data_type: DataType::Blob, constraints: vec![]},
            ],
            rows: vec![
                RowStack::new(Row(vec![Value::Integer(3), Value::Text("c_Jim".to_string()), Value::Real(3000.0), Value::Blob(b"0022".to_vec())])),
                RowStack::new(Row(vec![Value::Integer(1), Value::Text("a_John".to_string()), Value::Real(1000.0), Value::Blob(b"0000".to_vec())])),
                RowStack::new(Row(vec![Value::Null, Value::Null, Value::Null, Value::Null])),
                RowStack::new(Row(vec![Value::Integer(2), Value::Text("b_Jane".to_string()), Value::Real(2000.0), Value::Blob(b"0201".to_vec())])),
                RowStack::new(Row(vec![Value::Integer(3), Value::Text("b_Jim".to_string()), Value::Real(1500.0), Value::Blob(b"0102".to_vec())])),
                RowStack::new(Row(vec![Value::Integer(4), Value::Text("a_Jim".to_string()), Value::Real(500.0), Value::Blob(b"0101".to_vec())])),
                RowStack::new(Row(vec![Value::Integer(1), Value::Text("a_Jim".to_string()), Value::Real(5000.0), Value::Blob(b"0401".to_vec())])),
            ],
        }
    }
    // TODO: tests
}