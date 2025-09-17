use crate::db::table::alter_table;
use crate::db::table::create_table;
use crate::db::table::delete;
use crate::db::table::insert;
use crate::db::table::select;
use crate::db::table::update;
use crate::db::table::{Row, Table, drop_table};
use crate::interpreter::ast::SqlStatement;
use std::collections::HashMap;

pub struct Database {
    pub tables: HashMap<String, Table>,
    pub transaction: Option<TransactionLog>,
}

pub struct TransactionLog {
    pub entries: Vec<TransactionEntry>,
    pub savepoint_name: Vec<Savepoint>,
}

pub struct TransactionEntry {
    pub statement: SqlStatement,
    pub table_name: String,
    pub affected_rows: Vec<usize>,
}

pub struct Savepoint {
    pub name: String,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            transaction: None,
        }
    }

    pub fn execute(&mut self, sql_statement: SqlStatement) -> Result<Option<Vec<Row>>, String> {
        let sql_statement_clone = sql_statement.clone();
        return match sql_statement {
            SqlStatement::CreateTable(statement) => {
                create_table::create_table(self, statement)?;
                self.append_to_transaction(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::InsertInto(statement) => {
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_inserted = insert::insert(table, statement)?;
                self.append_to_transaction(sql_statement_clone, rows_inserted)?;
                Ok(None)
            }
            SqlStatement::Select(statement) => {
                let result = select::select_statement_stack(self, statement)?;
                Ok(Some(result))
            }
            SqlStatement::UpdateStatement(statement) => {
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_updated = update::update(table, statement)?;
                self.append_to_transaction(sql_statement_clone, rows_updated)?;
                Ok(None)
            }
            SqlStatement::DeleteStatement(statement) => {
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_deleted = delete::delete(table, statement)?;
                self.append_to_transaction(sql_statement_clone, rows_deleted)?;
                Ok(None)
            }
            SqlStatement::DropTable(statement) => {
                drop_table::drop_table(self, statement)?;
                self.append_to_transaction(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::AlterTable(statement) => {
                alter_table::alter_table(self, statement, self.transaction.is_some())?;
                self.append_to_transaction(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::BeginTransaction(_) => {
                self.transaction = Some(TransactionLog {
                    entries: vec![],
                    savepoint_name: vec![],
                });
                Ok(None)
            }
            SqlStatement::Commit => {
                if let Some(transaction) = self.transaction.take() {
                    for transaction_entry in transaction.entries.iter() {
                        let table = self.get_table_mut(transaction_entry.table_name.as_str())?;
                        table.commit_transaction(&transaction_entry.affected_rows)?;
                    }
                }
                Ok(None)
            }
            SqlStatement::Rollback(_) => {
                self.transaction = None;
                self.tables.iter_mut().for_each(|(_, table)| {
                    table.rollback_transaction();
                });
                Ok(None)
            }
            SqlStatement::Savepoint(statement) => {
                match &mut self.transaction {
                    Some(transaction) => {
                        transaction.savepoint_name.push(Savepoint {
                            name: statement.savepoint_name.clone(),
                        });
                    }
                    None => {
                        return Err("No transaction is currently active".to_string());
                    }
                }
                self.append_to_transaction(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::Release(statement) => {
                match &mut self.transaction {
                    Some(transaction) => {
                        transaction
                            .savepoint_name
                            .retain(|savepoint| savepoint.name != statement.savepoint_name);
                    }
                    None => {
                        return Err("No transaction is currently active".to_string());
                    }
                }
                Ok(None)
            }
        };
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    pub fn get_table(&self, table_name: &str) -> Result<&Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table `{}` does not exist", table_name));
        }
        Ok(self.tables.get(table_name).unwrap())
    }

    pub fn get_table_mut(&mut self, table_name: &str) -> Result<&mut Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table `{}` does not exist", table_name));
        }
        Ok(self.tables.get_mut(table_name).unwrap())
    }

    fn append_to_transaction(
        &mut self,
        sql_statement: SqlStatement,
        affected_rows: Vec<usize>,
    ) -> Result<(), String> {
        let table_name = match &sql_statement {
            SqlStatement::CreateTable(statement) => statement.table_name.clone(),
            SqlStatement::InsertInto(statement) => statement.table_name.clone(),
            SqlStatement::UpdateStatement(statement) => statement.table_name.clone(),
            SqlStatement::DeleteStatement(statement) => statement.table_name.clone(),
            SqlStatement::DropTable(statement) => statement.table_name.clone(),
            SqlStatement::AlterTable(statement) => statement.table_name.clone(),
            SqlStatement::Savepoint(_) => "".to_string(),
            _ => unreachable!(),
        };

        if let Some(transaction) = &mut self.transaction {
            transaction.entries.push(TransactionEntry {
                statement: sql_statement,
                table_name: table_name,
                affected_rows: affected_rows,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{ColumnDefinition, DataType};

    fn default_database() -> Database {
        Database {
            tables: HashMap::from([(
                "users".to_string(),
                Table::new(
                    "users".to_string(),
                    vec![
                        ColumnDefinition {
                            name: "id".to_string(),
                            data_type: DataType::Integer,
                            constraints: vec![],
                        },
                        ColumnDefinition {
                            name: "name".to_string(),
                            data_type: DataType::Text,
                            constraints: vec![],
                        },
                    ],
                ),
            )]),
            transaction: None,
        }
    }

    #[test]
    fn has_table_returns_proper_response() {
        let database = default_database();
        assert!(database.has_table("users"));
        assert!(!database.has_table("not_users"));
    }

    #[test]
    fn get_table_funcs_returns_proper_table() {
        let mut database = default_database();
        let table = database.get_table("users");
        assert!(table.is_ok());
        assert_eq!("users", table.unwrap().name);
        let table = database.get_table("not_users");
        assert!(table.is_err());
        assert_eq!("Table `not_users` does not exist", table.unwrap_err());
        let table = database.get_table_mut("users");
        assert!(table.is_ok());
        assert_eq!("users", table.unwrap().name);
        let table = database.get_table_mut("not_users");
        assert!(table.is_err());
        assert_eq!("Table `not_users` does not exist", table.unwrap_err());
    }
}
