#[cfg(test)]
use mollydb::db::table::Row;
#[cfg(test)]
use mollydb::interpreter::ast::OrderByDirection;
#[cfg(test)]
use std::cmp::Ordering;


#[cfg(test)]
pub fn assert_table_rows_eq_unordered(mut expected: Vec<Row>, mut actual: Vec<Row>) {
    expected.sort_by(|a, b| {
        let mut i = 0;
        while i < a.len() && i < b.len() && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal {
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
        while i < a.len() && i < b.len() && a[i].compare(&b[i], &OrderByDirection::Asc) == Ordering::Equal {
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