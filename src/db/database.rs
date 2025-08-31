use crate::db::table::{Table, Value};
use crate::cli::ast::{SqlStatement, CreateTableStatement, InsertIntoStatement, SelectStatement};
use crate::db::table::select;
use crate::db::table::insert;
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
        }
    }

    fn create_table(&mut self, statement: CreateTableStatement) -> Result<(), String> {
        if self.has_table(&statement.table_name) {
            return Err(format!("Table {} already exists", statement.table_name));
        }
        let table_name = statement.table_name;
        self.tables.insert(table_name.clone(), Table::new(table_name, statement.columns));
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
}