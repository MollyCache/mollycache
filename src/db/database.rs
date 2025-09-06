use crate::db::table::{Table, Value};
use crate::cli::ast::{SqlStatement, CreateTableStatement, InsertIntoStatement, SelectStatement, DeleteStatement};
use crate::db::table::select;
use crate::db::table::insert;
use crate::db::table::delete;
use std::collections::HashMap;

pub struct Database {
    tables: HashMap<String, Table>,
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
                let rows = self.select_from_table(statement)?;
                Ok(Some(rows))
            },
            SqlStatement::UpdateStatement(_statement) => {
                todo!();
            },
            SqlStatement::DeleteStatement(statement) => {
                self.delete_from_table(statement)?;
                Ok(None)
            },
        }
    }

    fn create_table(&mut self, statement: CreateTableStatement) -> Result<(), String> {
        if self.has_table(&statement.table_name) {
            return Err(format!("Table {} already exists", statement.table_name));
        }
        let table = Table::new(statement.table_name, statement.columns) ;
        self.tables.insert(table.name.clone(), table);
        Ok(())
    }

    fn insert_into_table(&mut self, statement: InsertIntoStatement) -> Result<(), String> {
        let table = self.get_table_mut(&statement.table_name)?;
        insert::insert(table, statement)?;
        Ok(())
    }

    fn select_from_table(&mut self, statement: SelectStatement) -> Result<Vec<Vec<Value>>, String> {
        let table = self.get_table(&statement.table_name)?;
        let rows = select::select(table, statement)?;
        Ok(rows)
    }

    fn delete_from_table(&mut self, statement: DeleteStatement) -> Result<(), String> {
        let table = self.get_table_mut(&statement.table_name)?;
        delete::delete(table, statement)?;
        Ok(())
    }

    fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    fn get_table(&self, table_name: &str) -> Result<&Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table {} does not exist", table_name));
        }
        Ok(self.tables.get(table_name).unwrap())
    }

    fn get_table_mut(&mut self, table_name: &str) -> Result<&mut Table, String> {
        if !self.has_table(table_name) {
            return Err(format!("Table {} does not exist", table_name));
        }
        Ok(self.tables.get_mut(table_name).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::CreateTableStatement;
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
    fn create_table_generates_proper_table() {
        let statement = CreateTableStatement {
            table_name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![] 
                },
            ],
        };
        let mut database = Database::new();
        assert!(database.create_table(statement).is_ok());
        assert!(database.has_table("users"));
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
        assert_eq!(table.unwrap_err(), "Table not_users does not exist");
        let table = database.get_table_mut("users");
        assert!(table.is_ok());
        assert_eq!(table.unwrap().name, "users");
        let table = database.get_table_mut("not_users");
        assert!(table.is_err());
        assert_eq!(table.unwrap_err(), "Table not_users does not exist");
    }
}