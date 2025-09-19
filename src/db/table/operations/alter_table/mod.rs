use crate::db::database::Database;
use crate::db::table::core::value::Value;
use crate::interpreter::ast::{AlterTableAction, AlterTableStatement};

pub fn alter_table(
    database: &mut Database,
    statement: AlterTableStatement,
    is_transaction: bool,
) -> Result<(), String> {
    return match statement.action {
        AlterTableAction::RenameTable { new_table_name } => {
            let mut table = database.pop_table_change(&statement.table_name)?;
            table.change_name(new_table_name.clone(), is_transaction);
            database.push_table_change(&new_table_name, table);
            Ok(())
        }
        AlterTableAction::RenameColumn {
            old_column_name,
            new_column_name,
        } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if !table.has_column(&old_column_name)? {
                return Err(format!(
                    "Column `{}` does not exist in table `{}`",
                    old_column_name, statement.table_name
                ));
            }
            let res =
                table
                    .columns
                    .rename_column(&old_column_name, &new_column_name, is_transaction);
            if res.is_err() {
                return Err(format!(
                    "Error renaming column: `{}` to `{}` in Table: `{}`",
                    old_column_name, new_column_name, statement.table_name
                ));
            }
            Ok(())
        }
        AlterTableAction::AddColumn { column_def } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if table.has_column(&column_def.name)? {
                return Err(format!(
                    "Column `{}` already exists in table `{}`",
                    column_def.name, statement.table_name
                ));
            }
            if is_transaction {
                table.get_row_stacks_mut().iter_mut().for_each(|row_stack| {
                    row_stack.append_clone();
                });
            }
            table.push_column(column_def, is_transaction);
            table.get_rows_mut().iter_mut().for_each(|row| {
                row.push(Value::Null);
            });
            Ok(())
        }
        AlterTableAction::DropColumn { column_name } => {
            let table = database.get_table_mut(&statement.table_name)?;
            if !table.has_column(&column_name)? {
                return Err(format!(
                    "Column `{}` does not exist in table `{}`",
                    column_name, statement.table_name
                ));
            }
            let index = table.columns.get_index_of_column(&column_name)?;
            let res = table.columns.drop_column(&column_name, is_transaction);
            if res.is_err() {
                return Err(format!(
                    "Error dropping column: `{}` from Table: `{}`",
                    column_name, statement.table_name
                ));
            }
            // This is kind of bad because it's an O(n^2) operation however SQLite
            // preserves the order of the columns after drop column statements.
            if is_transaction {
                table.get_row_stacks_mut().iter_mut().for_each(|row_stack| {
                    row_stack.append_clone();
                });
            }
            table.get_rows_mut().iter_mut().for_each(|row| {
                row.remove(index);
            });
            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::{
        column::ColumnDefinition,
        row::{Row, RowStack},
        value::DataType,
    };
    use crate::db::table::test_utils::default_database;

    #[test]
    fn alter_table_rename_table_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameTable {
                new_table_name: "new_users".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, false);
        assert!(result.is_ok());
        assert!(!database.has_table("users"));
        assert!(database.has_table("new_users"));
        assert!(database.get_table("new_users").unwrap().name().unwrap() == "new_users");
    }

    #[test]
    fn alter_table_rename_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameColumn {
                old_column_name: "name".to_string(),
                new_column_name: "new_name".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, false);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table_columns = table.unwrap().get_columns().unwrap();
        assert!(table_columns.iter().any(|column| column.name == "new_name"));
        assert!(table_columns.iter().any(|column| column.name == "new_name"));
    }

    #[test]
    fn alter_table_add_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::AddColumn {
                column_def: ColumnDefinition {
                    name: "new_column".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![],
                },
            },
        };
        let result = alter_table(&mut database, statement, false);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        let table_columns = table.get_columns().unwrap();
        assert!(table_columns.last().unwrap().name == "new_column");
        assert!(table_columns.len() == table[0].len());
        assert!(
            table
                .get_rows()
                .iter()
                .all(|row| row.last().unwrap() == &Value::Null)
        );
    }

    #[test]
    fn alter_table_drop_column_works_correctly() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::DropColumn {
                column_name: "age".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, false);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        let table_columns = table.get_columns().unwrap();
        assert!(!table_columns.iter().any(|column| column.name == "age"));
        let table_columns_len = table_columns.len();
        assert!(table_columns_len == table[0].len());
        let expected_columns_in_order = vec![
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
            ColumnDefinition {
                name: "money".to_string(),
                data_type: DataType::Real,
                constraints: vec![],
            },
        ];
        assert_eq!(
            expected_columns_in_order,
            table.get_columns_clone().unwrap()
        );
        let expected_rows = vec![
            Row(vec![
                Value::Integer(1),
                Value::Text("John".to_string()),
                Value::Real(1000.0),
            ]),
            Row(vec![
                Value::Integer(2),
                Value::Text("Jane".to_string()),
                Value::Real(2000.0),
            ]),
            Row(vec![
                Value::Integer(3),
                Value::Text("Jim".to_string()),
                Value::Real(3000.0),
            ]),
            Row(vec![Value::Integer(4), Value::Null, Value::Real(4000.0)]),
        ];
        assert_eq!(expected_rows, table.get_rows_clone());
    }

    #[test]
    fn alter_table_rename_column_works_correctly_with_transaction() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameColumn {
                old_column_name: "name".to_string(),
                new_column_name: "new_name".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, true);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        let table_columns = table.get_columns().unwrap();
        assert!(table_columns.iter().any(|column| column.name == "new_name"));
        let index_of_column = table.get_index_of_column(&"new_name".to_string()).unwrap();
        assert!(table.columns.stack.len() == 2);
        assert!(table.columns.stack[0][index_of_column].name == "name");
        assert!(table.columns.stack[1][index_of_column].name == "new_name");
    }

    #[test]
    fn alter_table_drop_column_works_correctly_with_transaction() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::DropColumn {
                column_name: "age".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, true);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        let table_columns = table.get_columns().unwrap();
        assert!(!table_columns.iter().any(|column| column.name == "age"));
        assert!(table.columns.stack.len() == 2);
        let expected_column_names = vec![
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
            ],
            vec!["id".to_string(), "name".to_string(), "money".to_string()],
        ];
        assert_eq!(
            expected_column_names,
            table
                .columns
                .stack
                .iter()
                .map(|column| column
                    .iter()
                    .map(|column| column.name.clone())
                    .collect::<Vec<String>>())
                .collect::<Vec<Vec<String>>>()
        );

        let expected_row_stacks = vec![
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(1),
                    Value::Text("John".to_string()),
                    Value::Integer(25),
                    Value::Real(1000.0),
                ]),
                Row(vec![
                    Value::Integer(1),
                    Value::Text("John".to_string()),
                    Value::Real(1000.0),
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(2),
                    Value::Text("Jane".to_string()),
                    Value::Integer(30),
                    Value::Real(2000.0),
                ]),
                Row(vec![
                    Value::Integer(2),
                    Value::Text("Jane".to_string()),
                    Value::Real(2000.0),
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(3),
                    Value::Text("Jim".to_string()),
                    Value::Integer(35),
                    Value::Real(3000.0),
                ]),
                Row(vec![
                    Value::Integer(3),
                    Value::Text("Jim".to_string()),
                    Value::Real(3000.0),
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(4),
                    Value::Null,
                    Value::Integer(40),
                    Value::Real(4000.0),
                ]),
                Row(vec![Value::Integer(4), Value::Null, Value::Real(4000.0)]),
            ]),
        ];
        assert_eq!(expected_row_stacks, table.get_row_stacks_clone());
    }

    #[test]
    fn alter_table_add_column_works_correctly_with_transaction() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::AddColumn {
                column_def: ColumnDefinition {
                    name: "new_column".to_string(),
                    data_type: DataType::Integer,
                    constraints: vec![],
                },
            },
        };
        let result = alter_table(&mut database, statement, true);
        assert!(result.is_ok());
        let table = database.get_table("users");
        assert!(table.is_ok());
        let table = table.unwrap();
        let table_columns = table.get_columns().unwrap();
        assert!(table_columns.last().unwrap().name == "new_column");
        assert!(table_columns.len() == table[0].len());
        assert!(table.columns.stack.len() == 2);
        let expected_column_names = vec![
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
            ],
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
                "new_column".to_string(),
            ],
        ];
        assert_eq!(
            expected_column_names,
            table
                .columns
                .stack
                .iter()
                .map(|column| column
                    .iter()
                    .map(|column| column.name.clone())
                    .collect::<Vec<String>>())
                .collect::<Vec<Vec<String>>>()
        );
        let expected_row_stacks = vec![
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(1),
                    Value::Text("John".to_string()),
                    Value::Integer(25),
                    Value::Real(1000.0),
                ]),
                Row(vec![
                    Value::Integer(1),
                    Value::Text("John".to_string()),
                    Value::Integer(25),
                    Value::Real(1000.0),
                    Value::Null,
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(2),
                    Value::Text("Jane".to_string()),
                    Value::Integer(30),
                    Value::Real(2000.0),
                ]),
                Row(vec![
                    Value::Integer(2),
                    Value::Text("Jane".to_string()),
                    Value::Integer(30),
                    Value::Real(2000.0),
                    Value::Null,
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(3),
                    Value::Text("Jim".to_string()),
                    Value::Integer(35),
                    Value::Real(3000.0),
                ]),
                Row(vec![
                    Value::Integer(3),
                    Value::Text("Jim".to_string()),
                    Value::Integer(35),
                    Value::Real(3000.0),
                    Value::Null,
                ]),
            ]),
            RowStack::new_with_stack(vec![
                Row(vec![
                    Value::Integer(4),
                    Value::Null,
                    Value::Integer(40),
                    Value::Real(4000.0),
                ]),
                Row(vec![
                    Value::Integer(4),
                    Value::Null,
                    Value::Integer(40),
                    Value::Real(4000.0),
                    Value::Null,
                ]),
            ]),
        ];
        assert_eq!(expected_row_stacks, table.get_row_stacks_clone());
    }

    #[test]
    fn alter_table_rename_table_works_correctly_with_transaction() {
        let mut database = default_database();
        let statement = AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameTable {
                new_table_name: "new_users".to_string(),
            },
        };
        let result = alter_table(&mut database, statement, true);
        assert!(result.is_ok());
        let table = database.get_table("new_users");
        assert!(table.is_ok());
        let expected_table_name_stack = vec!["users".to_string(), "new_users".to_string()];
        let table = table.unwrap();
        for (i, name) in expected_table_name_stack.iter().enumerate() {
            assert_eq!(*name, table.name.stack[i].clone());
        }
    }
}
