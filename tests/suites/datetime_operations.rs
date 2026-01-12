use mollycache::db::database::Database;
use mollycache::db::table::core::{row::Row, value::Value};
use mollycache::interpreter::run_sql;

use crate::common::assert_eq_table_rows;

#[test]
fn test_datetime_functions() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (id INTEGER);
    INSERT INTO users (id) VALUES (1);
    SELECT Date('2025-12-12 12:00:00') FROM users;
    SELECT Time('2025-12-12 12:00:00') FROM users;
    SELECT DateTime('2025-12-12 12:00:00') FROM users;
    SELECT JulianDay('2025-12-12 12:00:00') FROM users;
    SELECT UnixEpoch('2025-12-12 12:00:00') FROM users;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));

    let unix_result = result.pop().unwrap().unwrap().unwrap();
    assert!(matches!(unix_result[0].0[0], Value::Real(epoch) if epoch > 0.0));

    let jdn = match result.pop().unwrap().unwrap().unwrap()[0].0[0] {
        Value::Real(j) => j,
        _ => panic!("Expected Real value for JulianDay()"),
    };
    assert!((jdn - 2461022.0).abs() < 0.0001);

    let expected_datetime = vec![Row(vec![Value::Text("2025-12-12 12:00:00".to_string())])];
    assert_eq_table_rows(expected_datetime, result.pop().unwrap().unwrap().unwrap());

    let expected_time = vec![Row(vec![Value::Text("12:00:00".to_string())])];
    assert_eq_table_rows(expected_time, result.pop().unwrap().unwrap().unwrap());

    let expected_date = vec![Row(vec![Value::Text("2025-12-12".to_string())])];
    assert_eq_table_rows(expected_date, result.pop().unwrap().unwrap().unwrap());
}

#[test]
fn test_datetime_functions_with_modifiers() {
    let mut database = Database::new();
    let sql = "
    CREATE TABLE users (id INTEGER);
    INSERT INTO users (id) VALUES (1);
    SELECT Date('2025-12-12 12:00:00', '1 days') FROM users;
    SELECT DateTime('2025-12-12 12:00:00', '10 years') FROM users;
    SELECT DateTime('2025-12-12 12:00:00', '+0000-00-01 00:00:01') FROM users;
    ";
    let result = run_sql(&mut database, sql);
    assert_eq!(result.len(), 5);
    assert!(result[0].is_ok());
    assert!(result[1].is_ok());
    assert!(result[2].is_ok());
    assert!(result[3].is_ok());
    assert!(
        result[4].is_ok(),
        "Expected success for valid modifier '+0000-00-01 00:00:01'"
    );

    let expected_date = vec![Row(vec![Value::Text("2025-12-13".to_string())])];
    assert_eq_table_rows(
        expected_date,
        result[2].as_ref().unwrap().as_ref().unwrap().clone(),
    );
    let expected_datetime = vec![Row(vec![Value::Text("2035-12-12 12:00:00".to_string())])]; // Corrected behavior
    assert_eq_table_rows(
        expected_datetime,
        result[3].as_ref().unwrap().as_ref().unwrap().clone(),
    );
    let expected_mod_result = vec![Row(vec![Value::Text("2025-12-13 12:00:01".to_string())])];
    assert_eq_table_rows(
        expected_mod_result,
        result[4].as_ref().unwrap().as_ref().unwrap().clone(),
    );
}
