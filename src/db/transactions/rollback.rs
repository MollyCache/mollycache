use crate::db::database::Database;
use crate::db::transactions::{StatementEntry, TransactionEntry};
use crate::interpreter::ast::{AlterTableAction, RollbackStatement, SqlStatement};


pub fn rollback_statement(database: &mut Database, statement: &RollbackStatement) -> Result<(), String> {
    if !database.transaction.in_transaction() {
        return Err("No transaction is currently active".to_string());
    }
    
    if let Some(savepoint_name) = &statement.savepoint_name {
        // First make sure the savepoint exists
        if !database.transaction.savepoint_exists(savepoint_name)? {
            return Err(format!("Savepoint `{}` does not exist", savepoint_name));
        }
        // Rollback to savepoint - keep transaction active
        let mut current_entry = database.transaction.pop_entry()?;
        while current_entry.is_some() {
            match current_entry.unwrap() {
                TransactionEntry::Statement(transaction_statement) => {
                    rollback_transaction_entry(database, &transaction_statement)?;
                }
                TransactionEntry::Savepoint(savepoint_statement) => {
                    if savepoint_statement.name == *savepoint_name {
                        break;
                    }
                }
            }
            current_entry = database.transaction.pop_entry()?;
        }
    } else {
        // Full rollback - commit transaction to get entries and clear state
        if let Some(transaction_log) = database.transaction.commit_transaction()?.entries { // COMMIT TRANSACTIONS CLEARS THIS WITH TAKE
            for transaction_entry in transaction_log.iter().rev() {
                match transaction_entry {
                    TransactionEntry::Statement(statement) => {
                        // TODO: Some matching needs to be here for table based operations.
                        // CURRENTLY SUPPORTED STATEMENTS ARE:
                        // - ALTER TABLE RENAME COLUMN, ALTER TABLE ADD COLUMN, ALTER TABLE DROP COLUMN, ALTER TABLE RENAME TABLE
                        // - CREATE TABLE, DROP TABLE
                        // - INSERT INTO, UPDATE, DELETE
                        rollback_transaction_entry(database, &statement)?;
                    }
                    TransactionEntry::Savepoint(_) => {}
                }
            }
        }
    }
    Ok(())
}


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
            table.set_length(table.len() - statement_entry.affected_rows.len());
        }
        SqlStatement::UpdateStatement(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            for index in &statement_entry.affected_rows {
                table.get_row_stacks_mut()[*index].stack.pop();
            }
        }
        SqlStatement::DeleteStatement(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            table.set_length(table.len() + statement_entry.affected_rows.len());
        }
        _ => return Err("UNSUPPORTED".to_string()),
    }
    return Ok(());
}
