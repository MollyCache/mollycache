mod test_utils;

use mollycache::db::database::Database;
use mollycache::db::table::core::{row::Row, value::Value};
use mollycache::interpreter::run_sql;

use crate::test_utils::{assert_eq_run_sql, assert_eq_table_rows, assert_eq_table_rows_unordered};

#[test]
fn test_select_with_order_by_and_offset_exceeding_result_size() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    INSERT INTO users (id, name) VALUES (4, 'David');
    SELECT * FROM users ORDER BY id LIMIT 10 OFFSET 10;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // When offset exceeds the number of rows, should return empty result set
    // This is SQLite-compatible behavior
    let expected = vec![];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_select_with_order_by_and_offset_at_boundary() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    SELECT * FROM users ORDER BY id LIMIT 10 OFFSET 3;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // When offset equals the number of rows, should return empty result set
    let expected = vec![];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_select_with_order_by_and_partial_offset() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    INSERT INTO users (id, name) VALUES (4, 'David');
    SELECT * FROM users ORDER BY id LIMIT 10 OFFSET 2;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // Should return the last 2 rows
    let expected = vec![
        Row(vec![Value::Integer(3), Value::Text("Charlie".to_string())]),
        Row(vec![Value::Integer(4), Value::Text("David".to_string())]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_update_with_order_by_and_offset_exceeding_result_size() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    UPDATE users SET name = 'Updated' ORDER BY id LIMIT 1 OFFSET 10;
    SELECT * FROM users ORDER BY id;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // When offset exceeds the number of rows, should update 0 rows (SQLite-compatible behavior)
    let expected = vec![
        Row(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        Row(vec![Value::Integer(2), Value::Text("Bob".to_string())]),
        Row(vec![Value::Integer(3), Value::Text("Charlie".to_string())]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_delete_with_order_by_and_offset_exceeding_result_size() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    DELETE FROM users ORDER BY id LIMIT 1 OFFSET 10;
    SELECT * FROM users ORDER BY id;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // When offset exceeds the number of rows, should delete 0 rows (SQLite-compatible behavior)
    let expected = vec![
        Row(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        Row(vec![Value::Integer(2), Value::Text("Bob".to_string())]),
        Row(vec![Value::Integer(3), Value::Text("Charlie".to_string())]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_basic_statements_crud() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT,
        age INTEGER,
        money REAL
    );
    INSERT INTO users (id, name, age, money) VALUES (1, 'John', 25, 1000.0);
    INSERT INTO users (id, name, age, money) VALUES (2, 'Jane', 30, 2000.0);
    INSERT INTO users (id, name, age, money) VALUES (3, 'Jim', 35, 3000.0);
    UPDATE users SET money = 2000.0 WHERE id = 1;
    DELETE FROM users WHERE id = 2;
    SELECT * FROM users;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected = vec![
        Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
            Value::Integer(25),
            Value::Real(2000.0),
        ]),
        Row(vec![
            Value::Integer(3),
            Value::Text("Jim".to_string()),
            Value::Integer(35),
            Value::Real(3000.0),
        ]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
    assert!(
        result
            .into_iter()
            .all(|result| result.is_ok() && result.unwrap().is_none())
    );
}

#[test]
fn test_complex_statements_crud() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT,
        age INTEGER,
        money REAL
    );
    INSERT INTO users (id, name, age, money) VALUES (1, 'John', 25, 1500.0);
    INSERT INTO users (id, name, age, money) VALUES (2, 'Jane', 30, 2000.0);
    INSERT INTO users (id, name, age, money) VALUES 
        (3, 'Jim', 35, 3000.0), 
        (4, 'John', 70, 1000.0), 
        (5, Null, 80, Null); -- TODO: Restore id to NULL
    DELETE FROM users WHERE id >= 2 ORDER BY id LIMIT 1 OFFSET 2;
    SELECT name, age FROM users ORDER BY age DESC LIMIT 10 OFFSET 1;
    UPDATE users SET money = 1000.0 WHERE money IS NULL;
    SELECT age FROM users WHERE money = 1000.0;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected_first = vec![
        Row(vec![Value::Text("Jim".to_string()), Value::Integer(35)]),
        Row(vec![Value::Text("Jane".to_string()), Value::Integer(30)]),
        Row(vec![Value::Text("John".to_string()), Value::Integer(25)]),
    ];
    let expected_second = vec![Row(vec![Value::Integer(80)])];
    test_utils::assert_eq_table_rows_unordered(
        expected_second,
        result.pop().unwrap().unwrap().unwrap(),
    );
    assert!(result.pop().unwrap().unwrap().is_none());
    assert_eq_table_rows_unordered(expected_first, result.pop().unwrap().unwrap().unwrap());
    assert!(
        result
            .into_iter()
            .all(|result| result.is_ok() && result.unwrap().is_none())
    );
}

#[test]
fn test_parsing_errors() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE abc (
        id hello,
        name TEXT,
        age INTEGER,
        money REAL
    );
    SELECT * FROM users wherea; 
    SELECT * users;
    ";
    let result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_err()));
    let expected = vec![
        Err("Parsing Error: Error at line 3, column 11: Unexpected value: hello".to_string()),
        Err("Parsing Error: Error at line 8, column 24: Unexpected value: wherea".to_string()),
        Err("Parsing Error: Error at line 9, column 18: Unexpected value: ;".to_string()),
    ];
    assert_eq_run_sql(expected, result);
}

#[test]
fn test_execution_errors() {
    let mut database = Database::new();
    let sql = "
    SELECT * FROM users WHERE id = 'hello';
    ";
    let result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_err()));
    let expected = vec![Err(
        "Execution Error with statement starting on line 2 \n Error: Table `users` does not exist"
            .to_string(),
    )];
    assert_eq_run_sql(expected, result);
}

#[test]
fn test_drop_table() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    DROP TABLE users;
    DROP TABLE IF EXISTS users;
    DROP TABLE users;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);

    assert!(result[0].is_ok() && result[0].as_ref().unwrap().is_none());
    assert!(result[1].is_ok() && result[1].as_ref().unwrap().is_none());
    assert!(result[2].is_ok() && result[2].as_ref().unwrap().is_none());
    assert!(result[3].is_err());
    let expected_first =
        "Execution Error with statement starting on line 8 \n Error: Table `users` does not exist";
    assert_eq!(expected_first, result[3].as_ref().err().unwrap());

    assert!(result[4].is_err());
    let expected_second =
        "Execution Error with statement starting on line 9 \n Error: Table `users` does not exist";
    assert_eq!(expected_second, result[4].as_ref().err().unwrap());
}

#[test]
fn test_alter_table() {
    let mut database = Database::new();
    let sql = "
    /* These should all succeed */
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    ALTER TABLE users RENAME TO new_users;
    ALTER TABLE new_users RENAME COLUMN name TO new_name;
    ALTER TABLE new_users ADD COLUMN new_column INTEGER;
    ALTER TABLE new_users DROP COLUMN id;
    SELECT * FROM new_users;
    -- These should all fail
    ALTER TABLE users RENAME TO new_users;
    ALTER TABLE new_users DROP COLUMN id;
    ALTER TABLE new_users ADD COLUMN new_column INTEGER;
    ALTER TABLE new_users RENAME COLUMN id TO id_new;
    ";
    let result = run_sql(&mut database, sql);

    assert!(
        result[0..=5]
            .iter()
            .all(|result| result.is_ok() && result.as_ref().unwrap().is_none())
    );

    let expected = vec![Row(vec![Value::Text("John".to_string()), Value::Null])];
    let row = result[6].as_ref().unwrap().as_ref().unwrap();
    assert_eq_table_rows(expected, row.clone());

    let expected_errors = vec![
        "Execution Error with statement starting on line 14 \n Error: Table `users` does not exist",
        "Execution Error with statement starting on line 15 \n Error: Column `id` does not exist in table `new_users`",
        "Execution Error with statement starting on line 16 \n Error: Column `new_column` already exists in table `new_users`",
        "Execution Error with statement starting on line 17 \n Error: Column `id` does not exist in table `new_users`",
    ];

    assert!(result[7..=10].iter().all(|result| result.is_err()));
    assert_eq!(
        expected_errors,
        result[7..=10]
            .iter()
            .map(|result| result.as_ref().err().unwrap())
            .collect::<Vec<&String>>()
    );
}

#[test]
fn test_distinct_mode() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    INSERT INTO users (id, name) VALUES (2, 'Jane');
    INSERT INTO users (id, name) VALUES (3, 'Jim');
    INSERT INTO users (id, name) VALUES (4, 'John');
    SELECT DISTINCT name FROM users ORDER BY name ASC;
    ";
    let result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected = vec![
        Row(vec![Value::Text("Jane".to_string())]),
        Row(vec![Value::Text("Jim".to_string())]),
        Row(vec![Value::Text("John".to_string())]),
    ];
    let row = result[5].as_ref().unwrap().as_ref().unwrap();
    assert_eq_table_rows(expected, row.clone());
}

#[test]
fn test_distinct_with_limit_and_offset() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John');
    INSERT INTO users (id, name) VALUES (2, 'Jane');
    INSERT INTO users (id, name) VALUES (3, 'Jim');
    INSERT INTO users (id, name) VALUES (4, 'John');
    SELECT DISTINCT name FROM users ORDER BY name DESC LIMIT 1 OFFSET 1;
    ";
    let result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected = vec![Row(vec![Value::Text("Jim".to_string())])];
    let row = result[5].as_ref().unwrap().as_ref().unwrap();
    assert_eq_table_rows(expected, row.clone());
}

#[test]
fn test_select_clauses_with_literals() {
    let mut database = Database::new();
    let sql = "
    SELECT 1 FROM test_table; -- This should fail because table does not exist.
    CREATE TABLE test_table (
        id INTEGER,
        name TEXT
    );
    SELECT 'alice' FROM test_table; -- This should succeed and return nothing.
    INSERT INTO test_table (id, name) VALUES (1, 'John');
    SELECT name, 1.24 FROM test_table; -- This should succeed and return 1.24.
    INSERT INTO test_table (id, name) VALUES (2, 'Jane');
    SELECT X'1234', id, NULL FROM test_table; -- This should succeed and return two rows.
    ";
    let result = run_sql(&mut database, sql);
    let expected = vec![
        Err("Execution Error with statement starting on line 2 \n Error: Table `test_table` does not exist".to_string()),
        Ok(None),
        Ok(Some(vec![])),
        Ok(None),
        Ok(Some(vec![Row(vec![Value::Text("John".to_string()), Value::Real(1.24)])])),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Blob(vec![18, 52]), Value::Integer(1), Value::Null]),
            Row(vec![Value::Blob(vec![18, 52]), Value::Integer(2), Value::Null]),
        ])),
    ];
    assert_eq_run_sql(expected, result);
}

#[test]
fn test_division_by_zero_error() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE test_table (id INTEGER, value INTEGER);
    INSERT INTO test_table (id, value) VALUES (1, 10);
    INSERT INTO test_table (id, value) VALUES (2, 1);
    SELECT value / 0 FROM test_table WHERE id = 1;
    SELECT id / (value - 1) FROM test_table WHERE id = 2;
    SELECT value / (id - id) FROM test_table WHERE id = 1;
    ";
    let result = run_sql(&mut database, sql);

    // All division by zero operations should return errors, not panic
    assert!(
        result[3].is_err(),
        "Division by zero should return an error: {:?}",
        result[3]
    );
    assert!(
        result[4].is_err(),
        "Division by zero (1 - 1 = 0) should return an error: {:?}",
        result[4]
    );
    assert!(
        result[5].is_err(),
        "Division by zero (id - id = 0) should return an error: {:?}",
        result[5]
    );
}

#[test]
fn test_modulo_by_zero_error() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE test_table (id INTEGER, value INTEGER);
    INSERT INTO test_table (id, value) VALUES (1, 10);
    SELECT 10 % 0;
    SELECT value % 0 FROM test_table WHERE id = 1;
    ";
    let result = run_sql(&mut database, sql);

    // All modulo by zero operations should return errors, not panic
    assert!(result[2].is_err(), "Modulo by zero should return an error");
    assert!(result[3].is_err(), "Modulo by zero should return an error");
}

#[test]
fn test_update_with_offset_without_order_by() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    INSERT INTO users (id, name) VALUES (4, 'David');
    INSERT INTO users (id, name) VALUES (5, 'Eve');
    -- Update with OFFSET 2, LIMIT 2 (without ORDER BY)
    -- Should update rows at indices 2 and 3 (Charlie and David)
    UPDATE users SET name = 'Updated' LIMIT 2 OFFSET 2;
    SELECT * FROM users ORDER BY id;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // Rows 3 and 4 (Charlie and David) should have been updated
    let expected = vec![
        Row(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        Row(vec![Value::Integer(2), Value::Text("Bob".to_string())]),
        Row(vec![Value::Integer(3), Value::Text("Updated".to_string())]),
        Row(vec![Value::Integer(4), Value::Text("Updated".to_string())]),
        Row(vec![Value::Integer(5), Value::Text("Eve".to_string())]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}

#[test]
fn test_delete_with_offset_without_order_by() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'Alice');
    INSERT INTO users (id, name) VALUES (2, 'Bob');
    INSERT INTO users (id, name) VALUES (3, 'Charlie');
    INSERT INTO users (id, name) VALUES (4, 'David');
    INSERT INTO users (id, name) VALUES (5, 'Eve');
    -- Delete with OFFSET 3, LIMIT 1 (without ORDER BY)
    -- Should delete row at index 3 (David)
    DELETE FROM users LIMIT 1 OFFSET 3;
    SELECT * FROM users ORDER BY id;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    // Row 4 (David) should have been deleted
    let expected = vec![
        Row(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        Row(vec![Value::Integer(2), Value::Text("Bob".to_string())]),
        Row(vec![Value::Integer(3), Value::Text("Charlie".to_string())]),
        Row(vec![Value::Integer(5), Value::Text("Eve".to_string())]),
    ];
    assert_eq_table_rows(result.pop().unwrap().unwrap().unwrap(), expected);
}
