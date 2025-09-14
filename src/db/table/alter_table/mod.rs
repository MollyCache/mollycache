use crate::db::database::Database;
use crate::interpreter::ast::{AlterTableStatement, AlterTableAction};
use crate::db::table::Value;


pub fn alter_table(database: &mut Database, statement: AlterTableStatement) -> Result<(), String> {
    return match statement.action {
        AlterTableAction::RenameTable { new_table_name } => {
            let table = database.tables.remove(&statement.table_name);
            match table {
                Some(table) => database.tables.insert(new_table_name, table),
                None => return Err(format!("Table `{}` does not exist", statement.table_name)),
            };
            Ok(())
        }
        AlterTableAction::RenameColumn { old_column_name, new_column_name } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if !table.has_column(&old_column_name){
                return Err(format!("Column `{}` does not exist in table `{}`", old_column_name, statement.table_name));
            }
            table.columns.iter_mut().for_each(|column| {
                if column.name == old_column_name {
                    column.name = new_column_name.clone();
                }
            });
            Ok(())
        }
        AlterTableAction::AddColumn { column_def } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if table.has_column(&column_def.name){
                return Err(format!("Column `{}` already exists in table `{}`", column_def.name, statement.table_name));
            }
            table.columns.push(column_def);
            table.get_rows_mut().iter_mut().for_each(|row| {
                row.push(Value::Null);
            });
            Ok(())
        }
        AlterTableAction::DropColumn { column_name } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if !table.has_column(&column_name){
                return Err(format!("Column `{}` does not exist in table `{}`", column_name, statement.table_name));
            }
            let index = table.get_index_of_column(&column_name)?;
            // This is kind of bad because it's an O(n^2) operation however SQLite
            // preserves the order of the columns after drop column statements.
            table.columns.remove(index);
            table.get_rows_mut().iter_mut().for_each(|row| {
                row.remove(index);
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::test_utils::default_database;
    use crate::db::table::{ColumnDefinition, DataType, Row};

    #[test]
    fn alter_table_rename_table_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameTable { new_table_name: "new_users".to_string() },
        };
        let result = alter_table(&mut database, statement);
        assert!(result.is_ok());
        assert!(!database.tables.contains_key("users"));
        assert!(database.tables.contains_key("new_users"));
    }

    #[test]
    fn alter_table_rename_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameColumn { old_column_name: "name".to_string(), new_column_name: "new_name".to_string() },
        };
        let result = alter_table(&mut database, statement);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        assert!(table.unwrap().columns.iter().any(|column| column.name == "new_name"));
    }

    #[test]
    fn alter_table_add_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::AddColumn { column_def: ColumnDefinition { name: "new_column".to_string(), data_type: DataType::Integer, constraints: vec![] } },
        };
        let result = alter_table(&mut database, statement);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        assert!(table.columns.last().unwrap().name == "new_column");
        assert!(table.columns.len() == table[0].len());
        assert!(table.get_rows().iter().all(|row| row.last().unwrap() == &Value::Null));
    }

    #[test]
    fn alter_table_drop_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::DropColumn { column_name: "age".to_string() },
        };
        let result = alter_table(&mut database, statement);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        assert!(!table.columns.iter().any(|column| column.name == "age"));
        assert!(table.columns.len() == table[0].len());
        let expected_columns_in_order = 
        vec![
            ColumnDefinition {name: "id".to_string(), data_type: DataType::Integer, constraints: vec![]},
            ColumnDefinition {name: "name".to_string(), data_type: DataType::Text, constraints: vec![]},
            ColumnDefinition {name: "money".to_string(), data_type: DataType::Real, constraints: vec![]},
        ];
        assert_eq!( expected_columns_in_order, table.columns);
        let expected_rows = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Null, Value::Real(4000.0)]),
        ];
        assert_eq!(expected_rows, table.get_rows_clone());
    }
}