use crate::db::database::Database;
use crate::db::transactions::StatementEntry;
use crate::interpreter::ast::{AlterTableAction, SqlStatement};

pub fn rollback_transaction_entry(
    database: &mut Database,
    statement: &StatementEntry,
) -> Result<(), String> {
    match &statement.statement {
        SqlStatement::AlterTable(alter_table) => match alter_table.action {
            AlterTableAction::RenameColumn { .. } => {
                let table = database.get_table_mut(statement.table_name.as_str())?;
                table.rollback_columns();
            }
            AlterTableAction::AddColumn { .. } => {
                let table = database.get_table_mut(statement.table_name.as_str())?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::DropColumn { .. } => {
                let table = database.get_table_mut(statement.table_name.as_str())?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::RenameTable { ref new_table_name } => {
                // It is now under the new name
                let mut table = database
                    .tables
                    .remove(new_table_name.as_str())
                    .ok_or(format!("Table `{}` does not exist", new_table_name))?;
                table.last_mut().unwrap().rollback_name();
                database.tables.insert(table.last().unwrap().name()?.clone(), table);
            }
        },
        SqlStatement::Select(_) => {} // These should be kept in the log but obv do nothing.
        SqlStatement::CreateTable(_) => {
            database.tables.remove(statement.table_name.as_str());
        },
        _ => return Err("UNSUPPORTED".to_string()),
    }
    return Ok(());
}
