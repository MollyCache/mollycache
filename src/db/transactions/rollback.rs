use crate::db::table::core::table::Table;
use crate::db::transactions::StatementEntry;
use crate::interpreter::ast::{AlterTableAction, SqlStatement};

pub fn rollback_transaction_on_table(
    table: &mut Table,
    statement: &StatementEntry,
) -> Result<(), String> {
    match &statement.statement {
        SqlStatement::AlterTable(alter_table) => match alter_table.action {
            AlterTableAction::RenameColumn { .. } => {
                table.rollback_columns();
            }
            AlterTableAction::AddColumn { .. } => {
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::DropColumn { .. } => {
                table.rollback_columns();
                table.rollback_all_rows();
            }
            _ => return Err("UNSUPPORTED".to_string()),
        },
        SqlStatement::Select(_) => {} // These should be kept in the log but obv do nothing.
        _ => return Err("UNSUPPORTED".to_string()),
    }
    return Ok(());
}
