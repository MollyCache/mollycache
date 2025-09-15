mod test_utils;

use mollydb::db::database::Database;
use mollydb::db::table::{Row, Value};
use mollydb::interpreter::run_sql;

#[test]
fn test_set_operators() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John'), (2, 'Jane'), (3, 'Jim'), (4, 'Jack');
    SELECT name FROM users WHERE id = 1 UNION SELECT name FROM users WHERE id = 4;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected = vec![
        Row(vec![Value::Text("John".to_string())]),
        Row(vec![Value::Text("Jack".to_string())]),
    ];
    let row = result.pop().unwrap().unwrap().unwrap();
    test_utils::assert_table_rows_eq_unordered(expected, row);
    assert!(
        result
            .into_iter()
            .all(|result| result.is_ok() && result.unwrap().is_none())
    );
}

#[test]
fn test_set_operators_order_by_clauses_and_parentheses() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (
        id INTEGER,
        name TEXT
    );
    INSERT INTO users (id, name) VALUES (1, 'John'), (2, 'zane'), (3, 'Jane'), (4, 'Jack');
    (SELECT id, name FROM users WHERE id > 1 INTERSECT SELECT id, name FROM users WHERE id < 4)
    ORDER BY name ASC, id DESC;
    (SELECT id, name FROM users WHERE id > 1 INTERSECT SELECT id, name FROM users WHERE id < 4)
    ORDER BY name DESC LIMIT 1;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected_second = vec![
        Row(vec![Value::Integer(3), Value::Text("Jane".to_string())]),
        Row(vec![Value::Integer(2), Value::Text("zane".to_string())]),
    ];
    let expected_first = vec![Row(vec![
        Value::Integer(2),
        Value::Text("zane".to_string()),
    ])];
    assert_eq!(expected_first, result.pop().unwrap().unwrap().unwrap());
    assert_eq!(expected_second, result.pop().unwrap().unwrap().unwrap());
    assert!(
        result
            .into_iter()
            .all(|result| result.is_ok() && result.unwrap().is_none())
    );
}

#[test]
fn test_set_operators_with_different_tables_and_clause() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users1 (
        id INTEGER,
        name TEXT
    );
    CREATE TABLE users2 (
        id INTEGER,
        name TEXT
    );
    CREATE TABLE users3 (
        employee_id INTEGER,
        name TEXT
    );
    INSERT INTO users1 (id, name) VALUES (1, 'John'), (2, 'Jane'), (3, 'Jim'), (4, 'Jack');
    INSERT INTO users2 (id, name) VALUES (1, 'Fletcher'), (2, 'Jane'), (3, 'Jim'), (4, 'Fletcher');
    SELECT name FROM users1 UNION SELECT name FROM users2;
    SELECT * FROM users1 UNION SELECT * FROM users3;
    ";
    let mut result = run_sql(&mut database, sql);
    let first_result = result.pop().unwrap();
    assert!(first_result.is_err());
    let expected_second = "Execution Error with statement starting on line 17 \n Error: Columns mismatch between SELECT statements in Union".to_string();
    assert_eq!(expected_second, first_result.unwrap_err());
    assert!(result.iter().all(|result| result.is_ok()));
    let expected_first = vec![
        Row(vec![Value::Text("John".to_string())]),
        Row(vec![Value::Text("Fletcher".to_string())]),
        Row(vec![Value::Text("Jane".to_string())]),
        Row(vec![Value::Text("Jim".to_string())]),
        Row(vec![Value::Text("Jack".to_string())]),
    ];
    test_utils::assert_table_rows_eq_unordered(
        expected_first,
        result.pop().unwrap().unwrap().unwrap(),
    );
    assert!(
        result
            .into_iter()
            .all(|result| result.is_ok() && result.unwrap().is_none())
    );
}

// ADD TESTS with two seperate tables with different columns and using SELECT *
// two tables with same columns and using SELECT *
