use crate::cli::ast::{WhereStackElement};
use crate::db::table::{Table, Value};
use crate::db::table::select::where_condition::matches_where_clause;

// This file holds the logic for whether a row matches a where stack which is a vec of WhereConditions
// and logical operators stored in Reverse Polish Notation.
pub fn matches_where_stack(table: &Table, row: &Vec<Value>, where_stack: &Vec<WhereStackElement>) -> Result<bool, String> {
    let where_condition = match where_stack.first() {
        Some(WhereStackElement::Condition(where_condition)) => where_condition,
        _ => return Err(format!("Found nothing when expected edge")),
    };
    
    matches_where_clause(table, row, where_condition)
}