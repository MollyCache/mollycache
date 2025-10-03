use mollycache::db::table::core::row::Row;
use std::cmp::Ordering;

#[allow(dead_code)] // For some reason it can't pick up that this is used in integration tests. I'm prolly doing smth wrong.
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
    assert_eq!(expected, actual);
}
