use std::cmp::Ordering;

use crate::interpreter::ast::OrderByClause;
use crate::db::table::Table;
use crate::db::table::{Row};



// This sorting algorithm will always return a stable sort, this is given by all of the order columns
// then the input order of the rows is maintained with any required tie breaking.
pub fn get_ordered_row_indicies(table: &Table, mut row_indicies: Vec<usize>, order_by_clauses: &Vec<OrderByClause>) -> Result<Vec<usize>, String> {
    let columns: Vec<&String> = table.columns.iter().map(|column| &column.name).collect();
    row_indicies.sort_by(|a, b| {
        perform_comparions(&columns, &table[*a], &table[*b], order_by_clauses)
    });
    return Ok(row_indicies);
}

pub fn perform_comparions(columns: &Vec<&String>, row1: &Row, row2: &Row, order_by_clauses: &Vec<OrderByClause>) -> Ordering {
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
    
    #[test]
    fn get_ordered_rows_returns_rows_with_id_column_returns_rows_in_correct_order() {
        let table = default_table();
        let row_indicies: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
        let order_by_clauses = vec![OrderByClause {column: "id".to_string(), direction: OrderByDirection::Asc}];
        let result = get_ordered_row_indicies(&table, row_indicies, &order_by_clauses);
        assert!(result.is_ok());
        let expected = vec![2, 1, 6, 3, 0, 4, 5];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn get_ordered_rows_returns_rows_with_name_column_returns_rows_in_correct_order() {
        let table = default_table();
        let row_indicies: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
        let order_by_clauses = vec![OrderByClause {column: "name".to_string(), direction: OrderByDirection::Asc}];
        let result = get_ordered_row_indicies(&table, row_indicies, &order_by_clauses);
        assert!(result.is_ok());
        let expected = vec![2, 5, 6, 1, 3, 4, 0];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn get_ordered_rows_ordered_descending_returns_rows_in_correct_order() {
        let table = default_table();
        let row_indicies: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
        let order_by_clauses = vec![OrderByClause {column: "money".to_string(), direction: OrderByDirection::Desc}];
        let result = get_ordered_row_indicies(&table, row_indicies, &order_by_clauses);
        assert!(result.is_ok());
        let expected = vec![6, 0, 3, 4, 1, 5, 2];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn get_ordered_rows_multiple_sort_orders_returns_rows_in_correct_order() {  
        let table = default_table();
        let row_indicies: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
        let order_by_clauses = vec![OrderByClause {column: "name".to_string(), direction: OrderByDirection::Desc}, OrderByClause {column: "some_data".to_string(), direction: OrderByDirection::Asc}];
        let result = get_ordered_row_indicies(&table, row_indicies, &order_by_clauses);
        assert!(result.is_ok());
        let expected = vec![0, 4, 3, 1, 5, 6, 2];
        assert_eq!(expected, result.unwrap());
    }
}