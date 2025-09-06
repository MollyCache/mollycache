use crate::db::table::Table;
use crate::cli::ast::DeleteStatement;
use crate::db::table::helpers::common::get_initial_rows;


pub fn delete(table: &mut Table, statement: DeleteStatement) -> Result<(), String> {
    let _row_indexes_to_delete = get_row_indexes_to_delete(table, statement)?;
    Ok(())
}


fn get_row_indexes_to_delete(table: &mut Table, statement: DeleteStatement) -> Result<Vec<usize>, String> {
    let _rows = get_initial_rows(table, statement.where_clause, None)?;
    Ok(vec![])
}