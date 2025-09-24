use crate::db::database::Database;
use crate::db::transactions::{StatementEntry, TransactionEntry};
use crate::interpreter::ast::{AlterTableAction, RollbackStatement, SqlStatement};

pub fn rollback_statement(
    database: &mut Database,
    statement: &RollbackStatement,
) -> Result<(), String> {
    if !database.transaction.in_transaction() {
        return Err("No transaction is currently active".to_string());
    }

    if let Some(savepoint_name) = &statement.savepoint_name {
        // First make sure the savepoint exists
        if !database.transaction.savepoint_exists(savepoint_name)? {
            return Err(format!("Savepoint `{}` does not exist", savepoint_name));
        }
        // Rollback to savepoint - keep transaction active
        let mut current_entry = database.transaction.pop_entry()?;
        while current_entry.is_some() {
            match current_entry.unwrap() {
                TransactionEntry::Statement(transaction_statement) => {
                    rollback_transaction_entry(database, &transaction_statement)?;
                }
                TransactionEntry::Savepoint(savepoint_statement) => {
                    if savepoint_statement.name == *savepoint_name {
                        break;
                    }
                }
            }
            current_entry = database.transaction.pop_entry()?;
        }
    } else {
        // Full rollback - commit transaction to get entries and clear state
        if let Some(transaction_log) = database.transaction.commit_transaction()?.entries {
            // COMMIT TRANSACTIONS CLEARS THIS WITH TAKE
            for transaction_entry in transaction_log.iter().rev() {
                match transaction_entry {
                    TransactionEntry::Statement(statement) => {
                        // TODO: Some matching needs to be here for table based operations.
                        // CURRENTLY SUPPORTED STATEMENTS ARE:
                        // - ALTER TABLE RENAME COLUMN, ALTER TABLE ADD COLUMN, ALTER TABLE DROP COLUMN, ALTER TABLE RENAME TABLE
                        // - CREATE TABLE, DROP TABLE
                        // - INSERT INTO, UPDATE, DELETE
                        rollback_transaction_entry(database, &statement)?;
                    }
                    TransactionEntry::Savepoint(_) => {}
                }
            }
        }
    }
    Ok(())
}

pub fn rollback_transaction_entry(
    database: &mut Database,
    statement_entry: &StatementEntry,
) -> Result<(), String> {
    match &statement_entry.statement {
        SqlStatement::AlterTable(alter_table) => match alter_table.action {
            AlterTableAction::RenameColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
            }
            AlterTableAction::AddColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::DropColumn { .. } => {
                let table = database.get_table_mut(&statement_entry.table_name)?;
                table.rollback_columns();
                table.rollback_all_rows();
            }
            AlterTableAction::RenameTable { ref new_table_name } => {
                // It is now under the new name
                let mut table = database.pop_table_change(&new_table_name)?;
                table.rollback_name();
                database.push_table_change(&statement_entry.table_name, table);
            }
        },
        SqlStatement::Select(_) => {} // These should be kept in the log but obv do nothing.
        SqlStatement::CreateTable(_) => {
            database
                .tables
                .get_mut(&statement_entry.table_name)
                .unwrap()
                .pop();
        }
        SqlStatement::DropTable(_) => {
            // For drop table rollback, we need to pop the None that was pushed during the drop
            if let Some(table_versions) = database.tables.get_mut(&statement_entry.table_name) {
                table_versions.pop();
            }
        }
        SqlStatement::InsertInto(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            for _ in &statement_entry.affected_rows {
                table.get_row_stacks_mut().pop(); // We can pop all the rows off because they always get pushed to the end
            }
            table.set_length(table.len() - statement_entry.affected_rows.len());
        }
        SqlStatement::UpdateStatement(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            for index in &statement_entry.affected_rows {
                table.get_row_stacks_mut()[*index].stack.pop();
            }
        }
        SqlStatement::DeleteStatement(_) => {
            let table = database.get_table_mut(&statement_entry.table_name)?;
            table.set_length(table.len() + statement_entry.affected_rows.len());
        }
        _ => return Err("UNSUPPORTED".to_string()),
    }
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::{row::Row, value::Value};
    use crate::db::table::test_utils::{assert_table_rows_eq_unordered, default_database};
    use crate::db::transactions::{Savepoint, StatementEntry};
    use crate::interpreter::ast::{InsertIntoStatement, SqlStatement};

    #[test]
    fn test_rollback_statement_no_active_transaction() {
        let mut database = default_database();
        let statement = RollbackStatement {
            savepoint_name: None,
        };
        let result = rollback_statement(&mut database, &statement);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No transaction is currently active");
    }

    #[test]
    fn test_rollback_statement_with_savepoint() {
        let mut database = default_database();
        database.transaction.begin_transaction().unwrap();
        let savepoint = Savepoint {
            name: "test_savepoint".to_string(),
        };
        database.transaction.append_savepoint(savepoint).unwrap();
        let table = database.get_table_mut("users").unwrap();
        table.push(Row(vec![
            Value::Integer(5),
            Value::Text("test".to_string()),
            Value::Integer(50),
            Value::Real(5000.0),
        ]));
        let insert_statement = SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
            ]),
            values: vec![vec![
                Value::Integer(5),
                Value::Text("test".to_string()),
                Value::Integer(50),
                Value::Real(5000.0),
            ]],
        });
        database
            .transaction
            .append_entry(insert_statement, vec![4])
            .unwrap();
        assert_eq!(database.transaction.get_entries().unwrap().len(), 2);
        assert_eq!(database.get_table("users").unwrap().len(), 5);
        let rollback_stmt = RollbackStatement {
            savepoint_name: Some("test_savepoint".to_string()),
        };
        let result = rollback_statement(&mut database, &rollback_stmt);
        assert!(result.is_ok());
        assert!(database.transaction.in_transaction());
        assert_eq!(database.get_table("users").unwrap().len(), 4);
        assert_table_rows_eq_unordered(
            database.get_table("users").unwrap().get_rows_clone(),
            default_database()
                .get_table("users")
                .unwrap()
                .get_rows_clone(),
        );
    }

    #[test]
    fn test_rollback_transaction_entry_various_statements() {
        let mut database = default_database();
        let table = database.get_table_mut("users").unwrap();
        table.push(Row(vec![
            Value::Integer(5),
            Value::Text("test".to_string()),
            Value::Integer(50),
            Value::Real(5000.0),
        ]));
        let insert_statement = SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
            ]),
            values: vec![vec![
                Value::Integer(5),
                Value::Text("test".to_string()),
                Value::Integer(50),
                Value::Real(5000.0),
            ]],
        });
        let statement_entry = StatementEntry {
            statement: insert_statement,
            table_name: "users".to_string(),
            affected_rows: vec![4],
        };
        assert_eq!(table.len(), 5);
        let result = rollback_transaction_entry(&mut database, &statement_entry);
        assert!(result.is_ok());
        assert_eq!(database.get_table("users").unwrap().len(), 4);
        assert_table_rows_eq_unordered(
            database.get_table("users").unwrap().get_rows_clone(),
            default_database()
                .get_table("users")
                .unwrap()
                .get_rows_clone(),
        );
    }

    #[test]
    fn test_rollback_transaction_with_multiple_tables() {
        let mut database = default_database();
        database.tables.insert("orders".to_string(), vec![Some({
            let mut orders_table = crate::db::table::core::table::Table::new(
                "orders".to_string(),
                vec![
                    crate::db::table::core::column::ColumnDefinition {
                        name: "id".to_string(),
                        data_type: crate::db::table::core::value::DataType::Integer,
                        constraints: vec![],
                    },
                    crate::db::table::core::column::ColumnDefinition {
                        name: "user_id".to_string(),
                        data_type: crate::db::table::core::value::DataType::Integer,
                        constraints: vec![],
                    },
                    crate::db::table::core::column::ColumnDefinition {
                        name: "amount".to_string(),
                        data_type: crate::db::table::core::value::DataType::Real,
                        constraints: vec![],
                    },
                ],
            );
            orders_table.set_rows(vec![
                Row(vec![
                    Value::Integer(1),
                    Value::Integer(1),
                    Value::Real(100.0),
                ]),
                Row(vec![
                    Value::Integer(2),
                    Value::Integer(2),
                    Value::Real(200.0),
                ]),
            ]);
            orders_table
        })]);
        database.transaction.begin_transaction().unwrap();
        let users_table = database.get_table_mut("users").unwrap();
        users_table.push(Row(vec![
            Value::Integer(5),
            Value::Text("Alice".to_string()),
            Value::Integer(28),
            Value::Real(5000.0),
        ]));
        let users_insert = SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "money".to_string(),
            ]),
            values: vec![vec![
                Value::Integer(5),
                Value::Text("Alice".to_string()),
                Value::Integer(28),
                Value::Real(5000.0),
            ]],
        });
        database
            .transaction
            .append_entry(users_insert, vec![4])
            .unwrap();
        let orders_table = database.get_table_mut("orders").unwrap();
        orders_table.push(Row(vec![
            Value::Integer(3),
            Value::Integer(5),
            Value::Real(150.0),
        ]));
        let orders_insert = SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "orders".to_string(),
            columns: Some(vec![
                "id".to_string(),
                "user_id".to_string(),
                "amount".to_string(),
            ]),
            values: vec![vec![
                Value::Integer(3),
                Value::Integer(5),
                Value::Real(150.0),
            ]],
        });
        database
            .transaction
            .append_entry(orders_insert, vec![2])
            .unwrap();
        assert_eq!(database.get_table("users").unwrap().len(), 5);
        assert_eq!(database.get_table("orders").unwrap().len(), 3);
        assert_eq!(database.transaction.get_entries().unwrap().len(), 2);
        let rollback_stmt = RollbackStatement {
            savepoint_name: None,
        };
        let result = rollback_statement(&mut database, &rollback_stmt);
        assert!(result.is_ok());
        assert_eq!(database.get_table("users").unwrap().len(), 4);
        assert_eq!(database.get_table("orders").unwrap().len(), 2);
        assert!(!database.transaction.in_transaction());
        assert_table_rows_eq_unordered(
            database.get_table("users").unwrap().get_rows_clone(),
            default_database()
                .get_table("users")
                .unwrap()
                .get_rows_clone(),
        );
        let expected_orders = vec![
            Row(vec![
                Value::Integer(1),
                Value::Integer(1),
                Value::Real(100.0),
            ]),
            Row(vec![
                Value::Integer(2),
                Value::Integer(2),
                Value::Real(200.0),
            ]),
        ];
        assert_table_rows_eq_unordered(
            database.get_table("orders").unwrap().get_rows_clone(),
            expected_orders,
        );
    }
}
