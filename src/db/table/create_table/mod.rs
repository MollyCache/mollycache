use crate::db::database::Database;
use crate::interpreter::ast::{CreateTableStatement, ExistenceCheck};
use crate::db::table::Table;


pub fn create_table(database: &mut Database, statement: CreateTableStatement) -> Result<(), String> {
    if database.has_table(&statement.table_name) {
        match statement.existence_check {
            Some(ExistenceCheck::IfNotExists) => {
                return Ok(());
            }
            _ => {
                return Err(format!("Table {} already exists", statement.table_name));
            }
        }
    }
    let table = Table::new(statement.table_name, statement.columns) ;
    database.tables.insert(table.name.clone(), table);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::CreateTableStatement;
    use crate::db::table::{ColumnDefinition, DataType};
    use crate::db::table::test_utils::default_database;

    #[test]
    fn create_table_generates_proper_table() {
        let statement = CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![] 
                },
            ],
        };
        let mut database = Database::new();
        assert!(create_table(&mut database, statement).is_ok());
        assert!(database.has_table("users"));
    }

    #[test]
    fn create_table_errors_when_table_already_exists() {
        let statement = CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
            columns: vec![ColumnDefinition { name: "id".to_string(), data_type: DataType::Integer, constraints: vec![] }],
        };
        let mut database = default_database();
        let result = create_table(&mut database, statement);
        assert!(result.is_err());
        assert_eq!("Table users already exists", result.err().unwrap());
    }

    #[test]
    fn create_table_with_if_not_exists_clause_does_not_error_when_table_already_exists() {
        let statement = CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: Some(ExistenceCheck::IfNotExists),
            columns: vec![ColumnDefinition { name: "id".to_string(), data_type: DataType::Integer, constraints: vec![] }],
        };
        let mut database = default_database();
        let result = create_table(&mut database, statement);
        assert!(result.is_ok());
    }
}