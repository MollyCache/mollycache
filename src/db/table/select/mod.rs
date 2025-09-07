pub mod select_statement_clause;
use crate::db::database::Database;
use crate::db::table::Value;
use crate::interpreter::ast::SelectStatementStack;
use crate::interpreter::ast::SelectStatementStackElement;

pub fn select(database: &Database, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
    for select_statement in statement.elements {
        match select_statement {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                println!("Selecting from table: {:?}", select_statement);
                let table = database.get_table(&select_statement.table_name)?;
                let result = select_statement_clause::select_statement(table, select_statement);
                if result.is_err() {

                    return Err(result.unwrap_err());
                }
                else {
                    println!("Result: {:?}", result);
                    return result;
                }
            }
            _ => return Err(format!("Expected select statement, got {:?}", select_statement)),
        }
    }
    return Err("The statement stack is empty.".to_string());
}