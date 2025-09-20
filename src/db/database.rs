use crate::db::table::core::{row::Row, table::Table};
use crate::db::table::operations::{
    alter_table, create_table, delete, drop_table, insert, select, update,
};
use crate::db::transactions::TransactionLog;
use crate::db::transactions::{commit::commit_transaction, rollback::rollback_statement};
use crate::interpreter::ast::SqlStatement;
use std::collections::HashMap;

pub struct Database {
    pub tables: HashMap<String, Vec<Option<Table>>>,
    pub transaction: TransactionLog,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            transaction: TransactionLog { entries: None },
        }
    }

    pub fn execute(&mut self, sql_statement: SqlStatement) -> Result<Option<Vec<Row>>, String> {
        let sql_statement_clone = sql_statement.clone();
        return match sql_statement {
            SqlStatement::CreateTable(statement) => {
                create_table::create_table(self, statement, self.transaction.in_transaction())?;
                self.transaction.append_entry(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::InsertInto(statement) => {
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_inserted = insert::insert(table, statement)?;
                self.transaction
                    .append_entry(sql_statement_clone, rows_inserted)?;
                Ok(None)
            }
            SqlStatement::Select(statement) => {
                let result = select::select_statement_stack(self, statement)?;
                Ok(Some(result))
            }
            SqlStatement::UpdateStatement(statement) => {
                let is_transaction = self.transaction.in_transaction();
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_updated = update::update(table, statement, is_transaction)?;
                self.transaction
                    .append_entry(sql_statement_clone, rows_updated)?;
                Ok(None)
            }
            SqlStatement::DeleteStatement(statement) => {
                let is_transaction = self.transaction.in_transaction();
                let table = self.get_table_mut(&statement.table_name)?;
                let rows_deleted = delete::delete(table, statement, is_transaction)?;
                self.transaction
                    .append_entry(sql_statement_clone, rows_deleted)?;
                Ok(None)
            }
            SqlStatement::DropTable(statement) => {
                drop_table::drop_table(self, statement, self.transaction.in_transaction())?;
                self.transaction.append_entry(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::AlterTable(statement) => {
                alter_table::alter_table(self, statement, self.transaction.in_transaction())?;
                self.transaction.append_entry(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::BeginTransaction(_) => {
                self.transaction.begin_transaction();
                Ok(None)
            }
            SqlStatement::Commit => {
                commit_transaction(self)?;
                Ok(None)
            }
            SqlStatement::Rollback(statement) => {
                rollback_statement(self, &statement)?;
                Ok(None)
            }
            SqlStatement::Savepoint(_) => {
                self.transaction.append_entry(sql_statement_clone, vec![])?;
                Ok(None)
            }
            SqlStatement::Release(statement) => {
                self.transaction
                    .release_savepoint(&statement.savepoint_name)?;
                Ok(None)
            }
        };
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
            && !self.tables.get(table_name).is_none()
            && !self.tables.get(table_name).unwrap().is_empty()
            && self
                .tables
                .get(table_name)
                .unwrap()
                .last()
                .unwrap()
                .is_some()
    }

    pub fn get_table(&self, table_name: &str) -> Result<&Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table `{}` does not exist", table_name));
        }
        let table = self.tables.get(table_name).unwrap().last().unwrap();
        match table {
            Some(table) => Ok(table),
            _ => Err(format!("Table `{}` does not exist", table_name)),
        }
    }

    pub fn get_table_mut(&mut self, table_name: &str) -> Result<&mut Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table `{}` does not exist", table_name));
        }
        let table = self.tables.get_mut(table_name).unwrap().last_mut().unwrap();
        match table {
            Some(table) => Ok(table),
            _ => Err(format!("Table `{}` does not exist", table_name)),
        }
    }

    pub fn push_table_change(&mut self, table_name: &str, table: Table) {
        if !self.has_table(table_name) {
            self.tables
                .insert(table_name.to_string(), vec![Some(table)]);
        } else {
            self.tables.get_mut(table_name).unwrap().push(Some(table));
        }
    }

    pub fn pop_table_change(&mut self, table_name: &str) -> Result<Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table `{}` does not exist", table_name));
        }

        let table = self.tables.get_mut(table_name).unwrap().pop().unwrap();

        // Check if vector is empty before removing key
        let is_empty = self.tables.get(table_name).unwrap().is_empty();
        if is_empty {
            self.tables.remove(table_name);
        }

        match table {
            Some(table) => Ok(table),
            _ => Err(format!("Table `{}` does not exist", table_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::{column::ColumnDefinition, value::DataType};

    fn default_database() -> Database {
        Database {
            tables: HashMap::from([(
                "users".to_string(),
                vec![Some(Table::new(
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
                ))],
            )]),
            transaction: TransactionLog { entries: None },
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
        assert_eq!("users", table.unwrap().name().unwrap());
        let table = database.get_table("not_users");
        assert!(table.is_err());
        assert_eq!("Table `not_users` does not exist", table.unwrap_err());
        let table = database.get_table_mut("users");
        assert!(table.is_ok());
        assert_eq!("users", table.unwrap().name().unwrap());
        let table = database.get_table_mut("not_users");
        assert!(table.is_err());
        assert_eq!("Table `not_users` does not exist", table.unwrap_err());
    }
}
