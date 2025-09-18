use mollycache::db::table::core::row::Row;
use mollycache::interpreter::ast::OrderByDirection;
use std::cmp::Ordering;

#[allow(dead_code)] // For some reason it can't pick up that this is used in integration tests. I'm prolly doing smth wrong.
pub fn assert_eq_table_rows_unordered(mut expected: Vec<Row>, mut actual: Vec<Row>) {
    expected.sort_by(|a, b| {
        let mut i = 0;
        while i < a.len()
            && i < b.len()
            && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal
        {
            i += 1;
        }
        if i >= a.len() && i >= b.len() {
            Ordering::Equal
        } else if i >= a.len() {
            Ordering::Less
        } else if i >= b.len() {
            Ordering::Greater
        } else {
            a[i].compare(&b[i], &OrderByDirection::Asc)
        }
    });
    actual.sort_by(|a, b| {
        let mut i = 0;
        while i < a.len()
            && i < b.len()
            && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal
        {
            i += 1;
        }
        if i >= a.len() && i >= b.len() {
            Ordering::Equal
        } else if i >= a.len() {
            Ordering::Less
        } else if i >= b.len() {
            Ordering::Greater
        } else {
            a[i].compare(&b[i], &OrderByDirection::Asc)
        }
    });
    assert_eq!(expected, actual);
}
