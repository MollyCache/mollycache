use crate::db::database::Database;
use crate::interpreter::ast::{DropTableStatement, ExistenceCheck};

pub fn drop_table(
    database: &mut Database,
    statement: DropTableStatement,
    is_transaction: bool,
) -> Result<(), String> {
    if !database.has_table(&statement.table_name) {
        match statement.existence_check {
            Some(ExistenceCheck::IfExists) => {
                return Ok(());
            }
            _ => {
                return Err(format!("Table `{}` does not exist", statement.table_name));
            }
        }
    }
    if is_transaction {
        database
            .tables
            .get_mut(&statement.table_name)
            .unwrap()
            .push(None);
    } else {
        database.tables.remove(&statement.table_name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::test_utils::default_database;

    #[test]
    fn drop_table_drops_proper_table() {
        let statement = DropTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
        };
        let mut database = default_database();
        let result = drop_table(&mut database, statement, false);
        assert!(result.is_ok());
        assert!(!database.has_table("users"));
    }

    #[test]
    fn drop_table_errors_when_table_already_exists() {
        let statement = DropTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
        };
        let mut database = Database::new();
        let result = drop_table(&mut database, statement, false);
        assert!(result.is_err());
        assert_eq!("Table `users` does not exist", result.err().unwrap());
    }

    #[test]
    fn drop_table_with_if_exists_clause_does_not_error_when_table_already_exists() {
        let statement = DropTableStatement {
            table_name: "users".to_string(),
            existence_check: Some(ExistenceCheck::IfExists),
        };
        let mut database = Database::new();
        let result = drop_table(&mut database, statement, false);
        assert!(result.is_ok());
    }
}
