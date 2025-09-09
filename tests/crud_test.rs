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
        Err("Parsing Error: Error at line 9, column 13: Unexpected value: users".to_string()),
    ];
    assert_eq!(expected, result);
}

#[test]
fn test_execution_errors() {
    let mut database = Database::new();
    let sql = "
    SELECT * FROM users WHERE id = 'hello';
    ";
    let result = run_sql(&mut database, sql);
    assert!(result.iter().all(|result| result.is_err()));
    let expected = vec![Err("Execution Error with statement starting on line 2 \n Error: Table `users` does not exist".to_string())];
    assert_eq!(expected, result);
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
    let expected_first = "Execution Error with statement starting on line 8 \n Error: Table `users` does not exist";
    assert_eq!(expected_first, result[3].as_ref().err().unwrap());
    
    assert!(result[4].is_err());
    let expected_second = "Execution Error with statement starting on line 9 \n Error: Table `users` does not exist";
    assert_eq!(expected_second, result[4].as_ref().err().unwrap());
}

#[test]
fn test_alter_table() {
    let mut database = Database::new();
    let sql = "
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

    ALTER TABLE users RENAME TO new_users;
    ALTER TABLE new_users DROP COLUMN id;
    ALTER TABLE new_users ADD COLUMN new_column INTEGER;
    ALTER TABLE new_users RENAME COLUMN id TO id_new;
    ";
    let result = run_sql(&mut database, sql);

    assert!(result[0..=5].iter().all(|result| result.is_ok() && result.as_ref().unwrap().is_none()));

    let expected = vec![
        vec![Value::Text("John".to_string()), Value::Null],
    ];
    let row = result[6].as_ref().unwrap().as_ref().unwrap();
    assert_eq!(expected, *row);

    let expected_errors = vec![
        "Execution Error with statement starting on line 13 \n Error: Table `users` does not exist",
        "Execution Error with statement starting on line 14 \n Error: Column `id` does not exist in table `new_users`",
        "Execution Error with statement starting on line 15 \n Error: Column `new_column` already exists in table `new_users`",
        "Execution Error with statement starting on line 16 \n Error: Column `id` does not exist in table `new_users`",
    ];

    assert!(result[7..=10].iter().all(|result| result.is_err()));
    assert_eq!(expected_errors, result[7..=10].iter().map(|result| result.as_ref().err().unwrap()).collect::<Vec<&String>>());
}