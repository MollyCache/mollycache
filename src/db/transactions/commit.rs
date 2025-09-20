use crate::db::database::Database;
use crate::db::transactions::TransactionEntry;

pub fn commit_transaction(database: &mut Database) -> Result<(), String> {
    let transaction_log = database.transaction.commit_transaction()?;
    for transaction_entry in transaction_log.get_entries()?.iter() {
        match transaction_entry {
            TransactionEntry::Statement(statement) => {
                let table = database.get_table_mut(&statement.table_name)?;
                table.commit_transaction(&statement.affected_rows)?;
            }
            TransactionEntry::Savepoint(_) => {}
        }
    }
    Ok(())
}
