use std::cmp::Ordering;

use crate::interpreter::ast::OrderByClause;
use crate::db::table::{Row};

pub fn apply_order_by_from_precomputed<T: Clone>(to_order: &mut Vec<T>, precomputed: Vec<Row>, default: T, order_by_clause: &OrderByClause) -> () {
    let mut sorted_indices = (0..to_order.len()).collect::<Vec<usize>>();
    sorted_indices.sort_by(|a, b| perform_comparisons(&precomputed[*a], &precomputed[*b], order_by_clause));

    let sorted_vec: Vec<T> = sorted_indices.into_iter().map(|i| std::mem::replace(&mut to_order[i], default.clone())).collect();
    to_order.clear();
    to_order.extend(sorted_vec);
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

#[cfg(test)]
mod tests {
    use crate::db::table::{Table, Value, DataType, ColumnDefinition, Row, RowStack};

    fn _default_table() -> Table {
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