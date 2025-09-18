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
    println!("{:?}", result);
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
