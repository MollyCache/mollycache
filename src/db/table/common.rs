use crate::db::table::{Table, Value, DataType};

pub fn validate_and_clone_row(table: &Table, row: &Vec<Value>) -> Result<Vec<Value>, String> {
    if row.len() != table.width() {
        return Err(format!("Rows have incorrect width"));
    }

    let mut row_values: Vec<Value> = vec![];
    for (i, value) in row.iter().enumerate() {
        if value.get_type() != table.columns[i].data_type && value.get_type() != DataType::Null {
            return Err(format!("Data type mismatch for column {}", table.columns[i].name));
        }
        row_values.push(row[i].clone());
    }
    return Ok(row_values);
}