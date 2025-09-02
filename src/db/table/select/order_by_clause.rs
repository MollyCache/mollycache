use std::cmp::Ordering;

use crate::cli::ast::OrderByClause;
use crate::db::table::Table;
use crate::db::table::Value;
use crate::db::table::common::get_index_of_column;
use crate::cli::ast::OrderByDirection;

pub fn get_ordered_rows(table: &Table, rows: Vec<Vec<Value>>, order_by_clauses: &Vec<OrderByClause>) -> Result<Vec<Vec<Value>>, String> {
    let comparisons = order_by_clauses.iter().map(|order_by_clause| get_index_of_column(&table, &order_by_clause.column)).collect::<Result<Vec<usize>, String>>()?;
    let sorted_rows = sort_rows(table, rows, order_by_clauses, &comparisons);
    return sorted_rows;
}


pub fn sort_rows(table: &Table, rows: Vec<Vec<Value>>, order_by_clause: OrderByClause, comparisons: &Vec<usize>) -> Result<Vec<Vec<Value>>, String> {
    let mut null_rows = rows.iter().filter(|row| row[index] == Value::Null).collect::<Vec<&Vec<Value>>>();
    let mut non_null_rows = rows.iter().filter(|row| row[index] != Value::Null).collect::<Vec<&Vec<Value>>>();
    let sorted_rows = match order_by_clause.direction {
        OrderByDirection::Asc => {
            non_null_rows.sort_by(|a, b| perform_comparions(a, b, &comparisons));
            null_rows.extend(non_null_rows);
            null_rows
        },
        OrderByDirection::Desc => {
            non_null_rows.sort_by(|a, b| perform_comparions(a, b, &comparisons));
            non_null_rows.extend(null_rows);
            non_null_rows
        },
    };

    let mut result = vec![];
    for row in sorted_rows {
        result.push(row.clone());
    }
    return Ok(result);
}

fn perform_comparions(row1: &Vec<Value>, row2: &Vec<Value>, comparisons: &Vec<usize>) -> Ordering {
    let result = Ordering::Equal;
    for comparison in comparisons {
        let ordering = row1[*comparison].cmp(&row2[*comparison]);
        if ordering != Ordering::Equal {
            result = ordering;
            break;
        }
    }
    return result;

}