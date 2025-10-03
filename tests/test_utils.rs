use mollycache::db::table::core::row::Row;

#[allow(dead_code)] // For some reason it can't pick up that this is used in integration tests. I'm prolly doing smth wrong.
pub fn assert_eq_table_rows_unordered(mut expected: Vec<Row>, mut actual: Vec<Row>) {
    assert!(expected.len() == actual.len());
    expected.sort_by(|a, b| {
        let ordering = a.partial_cmp(b);
        assert!(ordering.is_some());
        ordering.unwrap()
    });
    actual.sort_by(|a, b| {
        let ordering = a.partial_cmp(b);
        assert!(ordering.is_some());
        ordering.unwrap()
    });
    assert_eq!(expected, actual);
}
