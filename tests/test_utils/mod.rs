use mollycache::db::table::core::row::Row;
use std::cmp::Ordering;

#[allow(dead_code)]
pub fn assert_eq_run_sql(
    expected: Vec<Result<Option<Vec<Row>>, String>>,
    actual: Vec<Result<Option<Vec<Row>>, String>>,
) {
    assert!(expected.len() == actual.len());
    for (first, second) in expected.iter().zip(actual.iter()) {
        match (first, second) {
            (Ok(Some(a)), Ok(Some(b))) => assert_eq_table_rows(a.clone(), b.clone()),
            (a, b) => assert!(a == b),
        }
    }
}

#[allow(dead_code)]
pub fn assert_eq_run_sql_unordered(
    expected: Vec<Result<Option<Vec<Row>>, String>>,
    actual: Vec<Result<Option<Vec<Row>>, String>>,
) {
    assert!(expected.len() == actual.len());
    for (first, second) in expected.iter().zip(actual.iter()) {
        match (first, second) {
            (Ok(Some(a)), Ok(Some(b))) => assert_eq_table_rows_unordered(a.clone(), b.clone()),
            (a, b) => assert!(a == b),
        }
    }
}

#[allow(dead_code)]
pub fn assert_eq_table_rows(expected: Vec<Row>, actual: Vec<Row>) {
    assert!(
        expected
            .into_iter()
            .zip(actual.into_iter())
            .all(|(e, a)| e.exactly_equal(&a))
    );
}

#[allow(dead_code)]
pub fn assert_eq_table_rows_unordered(mut expected: Vec<Row>, mut actual: Vec<Row>) {
    assert!(expected.len() == actual.len());
    expected.sort_by(|a, b| {
        for (first, second) in a.iter().zip(b.iter()) {
            let ordering = first.partial_cmp(second);
            assert!(ordering.is_some());
            if ordering.unwrap() != Ordering::Equal {
                return ordering.unwrap();
            }
        }
        Ordering::Equal
    });
    actual.sort_by(|a, b| {
        for (first, second) in a.iter().zip(b.iter()) {
            let ordering = first.partial_cmp(second);
            assert!(ordering.is_some());
            if ordering.unwrap() != Ordering::Equal {
                return ordering.unwrap();
            }
        }
        Ordering::Equal
    });
    assert_eq_table_rows(expected, actual);
}
