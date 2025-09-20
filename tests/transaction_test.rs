mod test_utils;

use mollycache::db::database::Database;
use mollycache::db::table::core::{row::Row, value::Value};
use mollycache::interpreter::run_sql;

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
    for (i, result) in result.iter().enumerate() {
        assert_eq!(expected[i], *result);
    }
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
    for (i, result) in result.iter().enumerate() {
        assert_eq!(expected[i], *result);
    }
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
    for (i, result) in result.iter().enumerate() {
        assert_eq!(expected[i], *result);
    }
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
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())])
        ])),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())])
        ])),
    ];
    for (i, result) in result.iter().enumerate() {
        assert_eq!(expected[i], *result);
    }
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
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("Jane".to_string())])])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Integer(1), Value::Text("John".to_string())])])),
    ];
    for (i, result) in result.iter().enumerate() {
        assert_eq!(expected[i], *result);
    }
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
        Ok(Some(vec![
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
        ])),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string())]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string())]),
            Row(vec![Value::Integer(4), Value::Text("Jill".to_string())]),
        ])),
    ];
    for (i, result) in result.iter().enumerate() {
        match (&expected[i], result) {
            (Ok(None), Ok(None)) => {
                assert_eq!(expected[i], *result);
            }
            (Ok(Some(expected_rows)), Ok(Some(actual_rows))) => {
                test_utils::assert_eq_table_rows_unordered(expected_rows.clone(), actual_rows.clone());
            }
            (_, _) => {
                assert_eq!(expected[i], *result);
            }
        }
    }
}