use crate::db::table::{drop_table, Table, Value};
use crate::interpreter::ast::{CreateTableStatement, DeleteStatement, DropTableStatement, InsertIntoStatement, SelectStatementStack, SqlStatement, UpdateStatement, AlterTableStatement};
use crate::db::table::select;
use crate::db::table::insert;
use crate::db::table::delete;
use crate::db::table::update;
use crate::db::table::create_table;
use std::collections::HashMap;

pub struct Database {
    pub tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, sql_statement: SqlStatement) -> Result<Option<Vec<Vec<Value>>>, String> {
        return match sql_statement {
            SqlStatement::CreateTable(statement) => {
                self.create_table(statement)?;
                Ok(None)
            },
            SqlStatement::InsertInto(statement) => {
                self.insert_into_table(statement)?;
                Ok(None)
            },
            SqlStatement::Select(statement) => {
                let result = self.select_statement_stack(statement)?;
                Ok(Some(result))
            },
            SqlStatement::UpdateStatement(statement) => {
                self.update_table(statement)?;
                Ok(None)
            },
            SqlStatement::DeleteStatement(statement) => {
                self.delete_from_table(statement)?;
                Ok(None)
            },
            SqlStatement::DropTable(statement) => {
                self.drop_table(statement)?;
                Ok(None)
            }
            SqlStatement::AlterTable(statement) => {
                self.alter_table(statement)?;
                Ok(None)
            }
        }
    }

    fn create_table(&mut self, statement: CreateTableStatement) -> Result<(), String> {
        create_table::create_table(self, statement)
    }

    fn insert_into_table(&mut self, statement: InsertIntoStatement) -> Result<(), String> {
        let table = self.get_table_mut(&statement.table_name)?;
        insert::insert(table, statement)
    }

    fn select_statement_stack(&mut self, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
        select::select_statement_stack(self, statement)
    }

    fn delete_from_table(&mut self, statement: DeleteStatement) -> Result<(), String> {
        let table = self.get_table_mut(&statement.table_name)?;
        delete::delete(table, statement)
    }

    fn update_table(&mut self, statement: UpdateStatement) -> Result<(), String> {
        let table = self.get_table_mut(&statement.table_name)?;
        update::update(table, statement)
    }

    fn drop_table(&mut self, statement: DropTableStatement) -> Result<(), String> {
        drop_table::drop_table(self, statement)
    }

    fn alter_table(&mut self, _statement: AlterTableStatement) -> Result<(), String> {
        todo!();
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    pub fn get_table(&self, table_name: &str) -> Result<&Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table not found: {}", table_name));
        }
        Ok(self.tables.get(table_name).unwrap())
    }

    fn get_table_mut(&mut self, table_name: &str) -> Result<&mut Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table not found: {}", table_name));
        }
        Ok(self.tables.get_mut(table_name).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{ColumnDefinition, DataType};


    fn default_database() -> Database {
        Database {
            tables: HashMap::from([
                ("users".to_string(), Table::new("users".to_string(), vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: DataType::Integer,
                        constraints: vec![]
                    },
                    ColumnDefinition {
                        name: "name".to_string(),
                        data_type: DataType::Text,
                        constraints: vec![]
                    },
                ]))
            ])
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
        assert_eq!(table.unwrap().name, "users");
        let table = database.get_table("not_users");
        assert!(table.is_err());
        assert_eq!(table.unwrap_err(), "Table not found: not_users");
        let table = database.get_table_mut("users");
        assert!(table.is_ok());
        assert_eq!(table.unwrap().name, "users");
        let table = database.get_table_mut("not_users");
        assert!(table.is_err());
        assert_eq!(table.unwrap_err(), "Table not found: not_users");
    }
}