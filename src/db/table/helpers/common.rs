use crate::db::table::{Table, Value, DataType};
use crate::cli::ast::{SelectStatement, SelectStatementColumns};
use crate::db::table::helpers::where_stack::matches_where_stack;

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

pub fn get_initial_rows(table: &Table, statement: &SelectStatement) -> Result<Vec<Vec<Value>>, String> {
    let mut rows: Vec<Vec<Value>> = vec![];
    if let Some(where_stack) = &statement.where_clause {
        for row in table.rows.iter() {
            if matches_where_stack(table, &row, &where_stack)? {
                rows.push(get_columns_from_row(table, &row, &statement.columns)?);
            }
        }
    } else {
        for row in table.rows.iter() {
            rows.push(get_columns_from_row(table, &row, &statement.columns)?);
        }
    }
    Ok(rows)
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