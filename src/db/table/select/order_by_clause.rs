use crate::cli::ast::OrderByClause;
use crate::db::table::Value;

pub fn get_ordered_rows(rows: Vec<Vec<Value>>, order_by_clause: &Vec<OrderByClause>) -> Result<Vec<Vec<Value>>, String> {
    Ok(rows)
}
