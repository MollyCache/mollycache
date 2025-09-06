use mollydb::db::database::Database;
use mollydb::interpreter::run_sql;
use mollydb::db::table::Value;

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
        vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(2000.0)],
        vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
    ];
    assert_eq!(result.pop().unwrap().unwrap().unwrap(), expected);
    assert!(result.into_iter().all(|result| result.is_ok() && result.unwrap().is_none()));
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
        (Null, Null, 80, Null);
    DELETE FROM users WHERE id >= 2 LIMIT 1 OFFSET 2;
    SELECT name, age FROM users ORDER BY age DESC LIMIT 10 OFFSET 1;
    UPDATE users SET money = 1000.0 WHERE money IS NULL;
    SELECT age FROM users WHERE money = 1000.0;
    ";
    let mut result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_ok()));
    let expected_first = vec![
        vec![Value::Text("Jim".to_string()), Value::Integer(35)],
        vec![Value::Text("Jane".to_string()), Value::Integer(30)],
        vec![Value::Text("John".to_string()), Value::Integer(25)],
    ];
    let expected_second = vec![
        vec![Value::Integer(80)],
    ];
    assert_eq!(expected_second, result.pop().unwrap().unwrap().unwrap());
    assert!(result.pop().unwrap().unwrap().is_none());
    assert_eq!(expected_first, result.pop().unwrap().unwrap().unwrap());
    assert!(result.into_iter().all(|result| result.is_ok() && result.unwrap().is_none()));
}