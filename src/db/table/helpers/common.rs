use std::collections::HashSet;

use crate::db::table::{Table, Value, DataType};
use crate::interpreter::ast::{SelectStatementColumns, WhereStackElement, OrderByClause, LimitClause};
use crate::db::table::helpers::where_clause::{matches_where_stack, WhereConditionEvaluator};
use crate::db::table::helpers::{order_by_clause::get_ordered_row_indicies, limit_clause::get_limited_rows};

pub struct DistinctOn<'a> {
    pub columns: &'a SelectStatementColumns,
}

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

pub fn get_row_columns_from_indicies(table: &Table, row_indicies: Vec<usize>, columns: Option<&SelectStatementColumns>) -> Result<Vec<Vec<Value>>, String> {
    let mut rows: Vec<Vec<Value>> = vec![];
    for index in row_indicies {
        let row = table.rows[index].clone();
        if let Some(columns) = columns {
            rows.push(get_columns_from_row(table, &row, columns)?);
        }
        else {
            rows.push(validate_and_clone_row(table, &row)?);
        }
    }
    Ok(rows)
}

pub fn get_row_indicies_matching_where_clause(table: &Table, where_clause: &Option<Vec<WhereStackElement>>) -> Result<Vec<usize>, String> {
    if let Some(where_clause) = where_clause {
        let mut row_indicies: Vec<usize> = vec![];
        for (i, row) in table.rows.iter().enumerate() {
            if matches_where_stack(table, &row, &where_clause, &WhereConditionEvaluator{})? {
                row_indicies.push(i);
            }
        }
        return Ok(row_indicies);
    }
    else {
        return Ok((0..table.rows.len()).collect());
    }
}

pub fn get_columns_from_row(table: &Table, row: &Vec<Value>, selected_columns: &SelectStatementColumns) -> Result<Vec<Value>, String> {
    let mut row_values: Vec<Value> = vec![];
    if *selected_columns == SelectStatementColumns::All {
        return Ok(validate_and_clone_row(table, row)?);
    } else {
        let specific_selected_columns = selected_columns.columns()?;
        for (i, column) in table.columns.iter().enumerate() {
            if (*specific_selected_columns).contains(&&column.name) {
                row_values.push(row[i].clone());
            }
        }
    }
    return Ok(row_values);
}

pub fn get_row_indicies_matching_clauses(table: &Table, mode: Option<DistinctOn>, where_clause: &Option<Vec<WhereStackElement>>, order_by_clause: &Option<Vec<OrderByClause>>, limit_clause: &Option<LimitClause>) -> Result<Vec<usize>, String> {
    let mut row_indicies = get_row_indicies_matching_where_clause(table, where_clause)?;

    if let Some(mode) = mode {
            row_indicies = remove_duplicate_rows_from_indicies(table, row_indicies, &mode.columns)?;

    }

    if let Some(order_by_clause) = order_by_clause {
        row_indicies = get_ordered_row_indicies(table, row_indicies, &order_by_clause)?;
    }

    if let Some(limit_clause) = limit_clause {
        let result = get_limited_rows(row_indicies, &limit_clause)?;
        return Ok(result.to_vec());
    }

    return Ok(row_indicies);
}

pub fn remove_duplicate_rows(rows: Vec<Vec<Value>>) -> Vec<Vec<Value>> {
    let set = rows.into_iter().collect::<HashSet<Vec<Value>>>();
    let result = set.into_iter().collect::<Vec<Vec<Value>>>();
    return result;
}

pub fn remove_duplicate_rows_from_indicies(table: &Table, mut row_indicies: Vec<usize>, columns: &SelectStatementColumns) -> Result<Vec<usize>, String> {
    let mut set = HashSet::new();
    let mut index = row_indicies.len();
    while index > 0 {
        index -= 1; 
        let row = get_columns_from_row(table, &table.rows[row_indicies[index]], columns)?;
        if set.contains(&row) {
            row_indicies.swap_remove(index);
        }
        else {
            set.insert(row);
        }
        
    }
    return Ok(row_indicies);
}