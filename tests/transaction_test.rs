mod test_utils;

use mollycache::db::database::Database;
use mollycache::db::table::core::{row::Row, value::Value};
use mollycache::interpreter::run_sql;

fn assert_result_unordered_eq(
    expected: Vec<Result<Option<Vec<Row>>, String>>,
    result: Vec<Result<Option<Vec<Row>>, String>>,
) {
    for (i, result) in result.iter().enumerate() {
        match (&expected[i], result) {
            (Ok(None), Ok(None)) => {
                assert_eq!(expected[i], *result);
            }
            (Ok(Some(expected_rows)), Ok(Some(actual_rows))) => {
                test_utils::assert_eq_table_rows_unordered(
                    expected_rows.clone(),
                    actual_rows.clone(),
                );
            }
            (_, _) => {
                assert_eq!(expected[i], *result);
            }
        }
    }
}

#[test]
fn test_transaction() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    SELECT * FROM users;
    BEGIN;
        ALTER TABLE users ADD COLUMN age INTEGER;
        SELECT * FROM users;
        ALTER TABLE users RENAME COLUMN age TO new_age;
        SELECT new_age FROM users;
        ALTER TABLE users DROP COLUMN name;
        SELECT * FROM users;
        ALTER TABLE users RENAME TO new_users;
        SELECT * FROM new_users;
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    SELECT * FROM new_users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
            Value::Null,
        ])])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Null])])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Null])])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Null])])),
        Err("Execution Error with statement starting on line 17 \n Error: Table `users` does not exist".to_string()),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Err("Execution Error with statement starting on line 20 \n Error: Table `new_users` does not exist".to_string()),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_create_table() {
    let mut database = Database::new();
    let sql = "
    BEGIN;
        CREATE TABLE users (
            id INTEGER,
            name TEXT
        );
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![])),
        Ok(None),
        Err("Execution Error with statement starting on line 9 \n Error: Table `users` does not exist".to_string()),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_drop_table() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    BEGIN;
        SELECT * FROM users;
        DROP TABLE users;
        SELECT * FROM users;
        CREATE TABLE users (
            id INTEGER,
            name TEXT
        );
        SELECT * FROM users;
        DROP TABLE users;
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),
        Ok(None),
        Err("Execution Error with statement starting on line 10 \n Error: Table `users` does not exist".to_string()),
        Ok(None),
        Ok(Some(vec![])),
        Ok(None),
        Err("Execution Error with statement starting on line 17 \n Error: Table `users` does not exist".to_string()),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_insert_into() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    SELECT * FROM users;
    BEGIN;
        INSERT INTO users (id, name) VALUES (2, 'Jane');
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
        ])),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_update() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    SELECT * FROM users;
    BEGIN;
        UPDATE users SET name = 'Jane' WHERE id = 1;
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("Jane".to_string()),
        ])])),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_delete() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John'), (2, 'Jane'), (3, 'Jim'), (4, 'Jill');
    SELECT * FROM users;
    BEGIN;
        DELETE FROM users WHERE id = 1;
        SELECT * FROM users;
        DELETE FROM users WHERE id >= 3;
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string())]),
            Row(vec![Value::Integer(4), Value::Text("Jill".to_string())]),
        ])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string())]),
            Row(vec![Value::Integer(4), Value::Text("Jill".to_string())]),
        ])),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(2),
            Value::Text("Jane".to_string()),
        ])])),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string())]),
            Row(vec![Value::Integer(4), Value::Text("Jill".to_string())]),
        ])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_savepoint() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    BEGIN;
        SAVEPOINT savepoint_name;
        INSERT INTO users (id, name) VALUES (1, 'John');
        SELECT * FROM users;
        ROLLBACK TO SAVEPOINT savepoint_name;
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(Some(vec![])),
        Ok(None),
        Ok(Some(vec![])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_rollback_to_savepoint_that_does_not_exist() {
    let mut database = Database::new();
    let sql = "
    BEGIN;
        SAVEPOINT savepoint_name;
        RELEASE SAVEPOINT savepoint_name;
        ROLLBACK TO SAVEPOINT savepoint_name;
    ROLLBACK;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Err("Execution Error with statement starting on line 5 \n Error: Savepoint `savepoint_name` does not exist".to_string()),
        Ok(None),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_commit_with_savepoint() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    BEGIN;
        SAVEPOINT savepoint_name;
        INSERT INTO users (id, name) VALUES (1, 'John');
        SELECT * FROM users;
        ROLLBACK TO SAVEPOINT savepoint_name;
        INSERT INTO users (id, name) VALUES (2, 'Jane');
        SELECT * FROM users;
    COMMIT;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(2),
            Value::Text("Jane".to_string()),
        ])])),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(2),
            Value::Text("Jane".to_string()),
        ])])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_commit_with_many_changes() {
    let mut database = Database::new();
    let sql = "
    BEGIN;
        CREATE TABLE users (
            id INTEGER,
            name TEXT
        );
        INSERT INTO users (id, name) VALUES (1, 'John');
        SAVEPOINT savepoint_name;
        DROP TABLE users;
        SELECT * FROM users;
        ROLLBACK TO SAVEPOINT savepoint_name;
        SELECT * FROM users;
        ALTER TABLE users ADD COLUMN age INTEGER;
        ALTER TABLE users RENAME COLUMN age TO new_age;
        SELECT * FROM users;
        ALTER TABLE users RENAME TO new_users;
    COMMIT;
    SELECT * FROM new_users;
    ";
    let result = run_sql(&mut database, sql);

    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Err("Execution Error with statement starting on line 10 \n Error: Table `users` does not exist".to_string()), 
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),  
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Null])])),    
        Ok(None),
        Err("Execution Error with statement starting on line 17 \n Error: Table `users` does not exist".to_string()),   
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Null])])), 
    ];

    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_with_multiple_savepoints() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    BEGIN;
        SAVEPOINT savepoint1;
        INSERT INTO users (id, name) VALUES (1, 'John');
        SAVEPOINT savepoint2;
        INSERT INTO users (id, name) VALUES (2, 'Jane');
        SELECT * FROM users;
        ROLLBACK TO SAVEPOINT savepoint2;
        SELECT * FROM users;
        ROLLBACK TO SAVEPOINT savepoint1;
        SELECT * FROM users;
    COMMIT;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
        ])),
        Ok(None),
        Ok(Some(vec![Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ])])),
        Ok(None),
        Ok(Some(vec![])),
        Ok(None),
        Ok(Some(vec![])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transaction_with_failed_operations() {
    let mut database = Database::new();
    let sql = "
    BEGIN;
        INSERT INTO users (id, name) VALUES (1, 'John');
    COMMIT;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Err("Execution Error with statement starting on line 3 \n Error: Table `users` does not exist".to_string()),
        Ok(None),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_incorrect_order_of_transaction_statements() {
    let mut database = Database::new();
    let sql = "
    COMMIT;
    ROLLBACK;
    BEGIN;
        BEGIN;
    COMMIT;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Err("Execution Error with statement starting on line 2 \n Error: No transaction is currently active".to_string()),
        Err("Execution Error with statement starting on line 3 \n Error: No transaction is currently active".to_string()),
        Ok(None),
        Err("Execution Error with statement starting on line 5 \n Error: Nested transactions are not allowed".to_string()),
        Ok(None),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_multiple_operations_on_the_same_row() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    BEGIN;
        INSERT INTO users (id, name) VALUES (1, 'John');
        UPDATE users SET name = 'Jane' WHERE id = 1;
        UPDATE users SET name = 'Jim' WHERE id = 1;
        INSERT INTO users (id, name) VALUES (2, 'Jane');
        SELECT * FROM users;
    ROLLBACK;
    SELECT * FROM users;

    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("Jim".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
        ])),
        Ok(None),
        Ok(Some(vec![])),
    ];
    assert_result_unordered_eq(expected, result);
}

#[test]
fn test_transactions_with_multiple_tables() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER
    );
    CREATE TABLE orders (
        user_id INTEGER
    );
    BEGIN;
        INSERT INTO users (id) VALUES (1);
        INSERT INTO orders (user_id) VALUES (1);
        SAVEPOINT savepoint_name;
        UPDATE users SET id = 2 WHERE id = 1;
        UPDATE orders SET user_id = 2 WHERE user_id = 1;
        DELETE FROM users WHERE id = 2;
        SELECT * FROM users;
        SELECT * FROM orders;
        ROLLBACK TO SAVEPOINT savepoint_name;
        SELECT * FROM users;
        SELECT * FROM orders;
    ROLLBACK;
    SELECT * FROM users;
    SELECT * FROM orders;
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(None),
        Ok(Some(vec![])),
        Ok(Some(vec![Row(vec![Value::Integer(2)])])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1)])])),
        Ok(Some(vec![Row(vec![Value::Integer(1)])])),
        Ok(None),
        Ok(Some(vec![])),
        Ok(Some(vec![])),
    ];
    assert_result_unordered_eq(expected, result);
}
