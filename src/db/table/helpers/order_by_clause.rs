use std::cmp::Ordering;

use crate::db::table::{Table, helpers::common::get_columns_from_row};
use crate::interpreter::ast::OrderByClause;
use crate::db::table::{Row};

pub fn apply_order_by(table: &Table, rows: &mut Vec<Row>, order_by_clause: &OrderByClause) -> Result<(), String> {
    let columns: Vec<Row> = rows.into_iter().enumerate().map(|row| {
        let columns = get_columns_from_row(table, row.1, &order_by_clause.columns)?;
        Ok(columns)
    }).collect::<Result<Vec<Row>, String>>()?;

    // TODO: there can be a much better way to do this using cycle permutations (minimize the amount of moving around)

    let mut sorted_indices = (0..rows.len()).collect::<Vec<usize>>();
    sorted_indices.sort_by(|a, b| perform_comparisons(&columns[*a], &columns[*b], order_by_clause));

    let sorted_rows: Vec<Row> = sorted_indices.into_iter().map(|i| std::mem::replace(&mut rows[i], Row(vec![]))).collect();
    rows.clear();
    rows.extend(sorted_rows);
    
    Ok(())
}

pub fn apply_order_by_from_indices(table: &Table, row_indices: &mut Vec<usize>, order_by_clause: &OrderByClause) -> Result<(), String> {
    let columns: Vec<Row> = row_indices.into_iter().map(|i| {
        let row = table.get(*i).ok_or("Invalid index".to_string())?;
        let columns = get_columns_from_row(table, row, &order_by_clause.columns)?;
        Ok(columns)
    }).collect::<Result<Vec<Row>, String>>()?;

    // TODO: there can be a much better way to do this using cycle permutations (minimize the amount of moving around)

    let mut sorted_indices = (0..row_indices.len()).collect::<Vec<usize>>();
    sorted_indices.sort_by(|a, b| perform_comparisons(&columns[*a], &columns[*b], order_by_clause));

    let sorted_row_indices: Vec<usize> = sorted_indices.into_iter().map(|i| std::mem::take(&mut row_indices[i])).collect();
    row_indices.clear();
    row_indices.extend(sorted_row_indices);

    Ok(())
}

fn perform_comparisons(row1: &Row, row2: &Row, order_by_clause: &OrderByClause) -> Ordering {
    // TODO: small optimization: only compute subsequent selectables if the previous ordering resulted in equality
    for (i, direction) in order_by_clause.directions.iter().enumerate() {
        let ordering =  row1[i].compare(&row2[i], direction);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    Ordering::Equal
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