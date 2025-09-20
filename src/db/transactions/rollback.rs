use crate::db::database::Database;
use crate::db::transactions::StatementEntry;
use crate::interpreter::ast::{AlterTableAction, SqlStatement};

pub fn rollback_transaction_entry(
    database: &mut Database,
    statement_entry: &StatementEntry,
) -> Result<(), String> {
    match &statement_entry.statement {
        SqlStatement::AlterTable(alter_table) => match alter_table.action {
            AlterTableAction::RenameColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
            }
            AlterTableAction::AddColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::DropColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::RenameTable { ref new_table_name } => {
                // It is now under the new name
                let mut table = database.pop_table_change(&new_table_name)?;
                table.rollback_name();
                database.push_table_change(&statement_entry.table_name, table);
            }
        },
        SqlStatement::Select(_) => {} // These should be kept in the log but obv do nothing.
        SqlStatement::CreateTable(_) => {
            database
                .tables
                .get_mut(&statement_entry.table_name)
                .unwrap()
                .pop();
        }
        SqlStatement::DropTable(_) => {
            // For drop table rollback, we need to pop the None that was pushed during the drop
            if let Some(table_versions) = database.tables.get_mut(&statement_entry.table_name) {
                table_versions.pop();
            }
        }
        SqlStatement::InsertInto(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            for _ in &statement_entry.affected_rows {
                table.get_row_stacks_mut().pop(); // We can pop all the rows off because they always get pushed to the end
            }
        }
        _ => return Err("UNSUPPORTED".to_string()),
    }
    return Ok(());
}
