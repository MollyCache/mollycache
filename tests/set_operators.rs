mod test_utils;

use mollydb::db::database::Database;
use mollydb::interpreter::run_sql;
use mollydb::db::table::Value;

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
        vec![Value::Text("John".to_string())],
        vec![Value::Text("Jack".to_string())],
    ];
    test_utils::assert_table_rows_eq_unordered(expected, result.pop().unwrap().unwrap().unwrap());
    assert!(result.into_iter().all(|result| result.is_ok() && result.unwrap().is_none()));
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
        vec![Value::Integer(3), Value::Text("Jane".to_string())],
        vec![Value::Integer(2), Value::Text("zane".to_string())],
    ];
    let expected_first = vec![
        vec![Value::Integer(2), Value::Text("zane".to_string())],
    ];
    test_utils::assert_table_rows_eq_unordered(expected_first, result.pop().unwrap().unwrap().unwrap());
    test_utils::assert_table_rows_eq_unordered(expected_second, result.pop().unwrap().unwrap().unwrap());
    assert!(result.into_iter().all(|result| result.is_ok() && result.unwrap().is_none()));
}

// ADD TESTS with two seperate tables with different columns and using SELECT *
// two tables with same columns and using SELECT *