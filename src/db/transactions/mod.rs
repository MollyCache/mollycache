use crate::interpreter::ast::SqlStatement;
pub mod rollback;

#[derive(Debug, PartialEq, Clone)]
pub struct TransactionLog {
    pub entries: Option<Vec<TransactionEntry>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionEntry {
    Statement(StatementEntry),
    Savepoint(Savepoint),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StatementEntry {
    pub statement: SqlStatement,
    pub table_name: String,
    pub affected_rows: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Savepoint {
    pub name: String,
}

impl TransactionLog {
    pub fn in_transaction(&self) -> bool {
        self.entries.is_some()
    }

    pub fn append_entry(
        &mut self,
        sql_statement: SqlStatement,
        affected_rows: Vec<usize>,
    ) -> Result<(), String> {
        if !self.in_transaction() {
            return Ok(());
        }
        let table_name = match &sql_statement {
            SqlStatement::CreateTable(statement) => statement.table_name.clone(),
            SqlStatement::InsertInto(statement) => statement.table_name.clone(),
            SqlStatement::UpdateStatement(statement) => statement.table_name.clone(),
            SqlStatement::DeleteStatement(statement) => statement.table_name.clone(),
            SqlStatement::DropTable(statement) => statement.table_name.clone(),
            SqlStatement::AlterTable(statement) => statement.table_name.clone(),
            SqlStatement::Savepoint(statement) => {
                self.append_savepoint(Savepoint {
                    name: statement.savepoint_name.clone(),
                })?;
                return Ok(());
            }
            _ => return Err("Invalid transaction entry".to_string()),
        };
        self.get_entries_mut()?
            .push(TransactionEntry::Statement(StatementEntry {
                statement: sql_statement,
                table_name: table_name,
                affected_rows: affected_rows,
            }));
        Ok(())
    }

    pub fn append_savepoint(&mut self, savepoint: Savepoint) -> Result<(), String> {
        self.get_entries_mut()?
            .push(TransactionEntry::Savepoint(savepoint));
        Ok(())
    }

    pub fn release_savepoint(&mut self, savepoint_name: &String) -> Result<(), String> {
        self.get_entries_mut()?.retain(|entry| match entry {
            TransactionEntry::Savepoint(savepoint) => savepoint.name != *savepoint_name,
            _ => true,
        });
        Ok(())
    }

    pub fn savepoint_exists(&self, savepoint_name: &String) -> Result<bool, String> {
        Ok(self.get_entries()?.iter().any(|entry| match entry {
            TransactionEntry::Savepoint(savepoint) => savepoint.name == *savepoint_name,
            _ => false,
        }))
    }

    pub fn begin_transaction(&mut self) {
        self.entries = Some(vec![]);
    }

    pub fn commit_transaction(&mut self) -> Result<TransactionLog, String> {
        let transaction_log = TransactionLog {
            entries: self.entries.take(),
        };
        self.entries = None;
        Ok(transaction_log)
    }

    pub fn pop_entry(&mut self) -> Result<Option<TransactionEntry>, String> {
        Ok(self.get_entries_mut()?.pop())
    }

    pub fn get_entries(&self) -> Result<&Vec<TransactionEntry>, String> {
        self.entries
            .as_ref()
            .ok_or_else(|| "No transaction is currently active".to_string())
    }

    fn get_entries_mut(&mut self) -> Result<&mut Vec<TransactionEntry>, String> {
        self.entries
            .as_mut()
            .ok_or_else(|| "No transaction is currently active".to_string())
    }
}
