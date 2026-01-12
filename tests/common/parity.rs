use mollycache::db::database::Database;
use mollycache::db::table::core::row::Row;
use mollycache::db::table::core::value::Value;
use mollycache::interpreter::run_sql;
use rusqlite::Connection;
use rusqlite::types::ValueRef;

pub struct ParityManager {
    molly_db: Database,
    sqlite_conn: Connection,
}

impl ParityManager {
    pub fn new() -> Self {
        let molly_db = Database::new();
        let sqlite_conn =
            Connection::open_in_memory().expect("Failed to create in-memory SQLite DB");
        Self {
            molly_db,
            sqlite_conn,
        }
    }

    // Expects the test to provide a single statement (or one that rusqlite prepare will handle correctly).
    pub fn assert_parity_query(&mut self, sql: &str) {
        let molly_result = run_sql(&mut self.molly_db, sql);

        // SQLite
        // We assume `sql` is one statement for this method.
        let sqlite_result = match self.sqlite_conn.prepare(sql) {
            Ok(mut stmt) => {
                if stmt.column_count() > 0 {
                    let column_count = stmt.column_count();
                    let mut rows = stmt.query([]).expect("SQLite query failed");
                    let mut result_rows = Vec::new();
                    while let Some(row) = rows.next().expect("SQLite next failed") {
                        let mut row_values = Vec::new();
                        for i in 0..column_count {
                            let val_ref = row.get_ref(i).unwrap();
                            row_values.push(sqlite_val_to_molly_val(val_ref));
                        }
                        result_rows.push(Row(row_values));
                    }
                    Ok(Some(result_rows))
                } else {
                    match stmt.execute([]) {
                        Ok(_) => Ok(None),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
            Err(e) => Err(e.to_string()),
        };

        // Compare
        // Molly might return multiple results if the string had multiple statements.
        // We asserted this method is for one statement.
        // We check the LAST result from Molly if there are multiple (e.g. comments + stmt),
        // or just the first.
        let molly_res = molly_result.last().unwrap().clone();

        match (molly_res, sqlite_result) {
            (Ok(Some(m_rows)), Ok(Some(s_rows))) => {
                assert_rows_equal(s_rows, m_rows);
            }
            (Ok(None), Ok(None)) => {
                // Both executed OK with no rows (e.g. INSERT)
            }
            (Err(_), Err(_)) => {
                // Both failed
            }
            (Ok(None), Ok(Some(_))) => panic!("Molly returned no rows, SQLite returned rows"),
            (Ok(Some(_)), Ok(None)) => panic!("Molly returned rows, SQLite returned no rows"),
            (Ok(_), Err(e)) => panic!("Molly succeeded, SQLite failed: {}", e),
            (Err(e), Ok(_)) => panic!("Molly failed: {}, SQLite succeeded", e),
        }
    }
}

fn assert_rows_equal(expected: Vec<Row>, actual: Vec<Row>) {
    assert_eq!(expected.len(), actual.len(), "Row count mismatch");
    for (i, (exp_row, act_row)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(
            exp_row.0.len(),
            act_row.0.len(),
            "Column count mismatch at row {}",
            i
        );
        for (j, (exp_val, act_val)) in exp_row.0.iter().zip(act_row.0.iter()).enumerate() {
            if !values_equal_approx(exp_val, act_val) {
                panic!(
                    "Value mismatch at row {}, col {}: expected {:?}, got {:?}",
                    i, j, exp_val, act_val
                );
            }
        }
    }
}

fn values_equal_approx(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Real(f1), Value::Real(f2)) => (f1 - f2).abs() < 1e-9,
        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
        (Value::Text(t1), Value::Text(t2)) => t1 == t2,
        (Value::Blob(b1), Value::Blob(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        // Allow cross-type comparison for int/real if they are close (SQLite might return int for 1.0)
        (Value::Integer(i), Value::Real(f)) => (*i as f64 - f).abs() < 1e-9,
        (Value::Real(f), Value::Integer(i)) => (f - *i as f64).abs() < 1e-9,
        _ => false,
    }
}

fn sqlite_val_to_molly_val(val: ValueRef) -> Value {
    match val {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Integer(i),
        ValueRef::Real(f) => Value::Real(f),
        ValueRef::Text(t) => Value::Text(String::from_utf8_lossy(t).to_string()),
        ValueRef::Blob(b) => Value::Blob(b.to_vec()),
    }
}
