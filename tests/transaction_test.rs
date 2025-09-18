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
    ROLLBACK;
    SELECT * FROM users;
    ";
    let result = run_sql(&mut database, sql);
    println!("{:?}", result);
    let expected = vec![
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
        ])),
        Ok(None),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Null]),
        ])),
        Ok(None),
        Ok(Some(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string())]),
        ])),
    ];
    for (i, result) in result.iter().enumerate() {
        assert!(result.is_ok());
        assert_eq!(expected[i], *result);
    }
}