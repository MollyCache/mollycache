pub mod select_statement;
use crate::db::database::Database;
use crate::db::table::Value;
use crate::interpreter::ast::SelectStatementStack;
use crate::interpreter::ast::SelectStatementStackElement;


pub fn select_statement_stack(database: &Database, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
    let select_statement = statement.elements.first();
    if let Some(select_statement) = select_statement {
        match select_statement {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                let rows = select_statement::select_statement(database.get_table(&select_statement.table_name)?, select_statement);
                return rows;
            }
            _ => Err(format!("Expected select statement, got {:?}", select_statement)),
        }
    } else {
        Ok(vec![])
    }
}