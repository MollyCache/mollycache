use std::cmp::Ordering;

use crate::db::table::core::row::Row;
use crate::interpreter::ast::OrderByClause;

pub fn apply_order_by_from_precomputed<T: Clone>(
    to_order: &mut Vec<T>,
    precomputed: Vec<Row>,
    default: T,
    order_by_clause: &OrderByClause,
) -> () {
    let mut sorted_indices = (0..to_order.len()).collect::<Vec<usize>>();
    sorted_indices
        .sort_by(|a, b| perform_comparisons(&precomputed[*a], &precomputed[*b], order_by_clause));

    let sorted_vec: Vec<T> = sorted_indices
        .into_iter()
        .map(|i| std::mem::replace(&mut to_order[i], default.clone()))
        .collect();
    to_order.clear();
    to_order.extend(sorted_vec);
}

fn perform_comparisons(row1: &Row, row2: &Row, order_by_clause: &OrderByClause) -> Ordering {
    // TODO: small optimization: only compute subsequent selectables if the previous ordering resulted in equality
    // Fletcher - I think this is done here? Can this comment be removed?
    for (i, direction) in order_by_clause.directions.iter().enumerate() {
        let ordering = row1[i].compare(&row2[i], direction);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::{row::Row, value::Value};
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::SelectableStack;
    use crate::interpreter::ast::SelectableStackElement;

    #[test]
    fn apply_order_by_from_precomputed_single_column_asc() {
        let mut to_order = vec!["second", "fourth", "third", "first"];

        let precomputed = vec![
            Row(vec![Value::Integer(25)]),
            Row(vec![Value::Integer(55)]),
            Row(vec![Value::Integer(35)]),
            Row(vec![Value::Integer(22)]),
        ];

        let order_by_clause = OrderByClause {
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("age".to_string())],
            },
            column_names: vec!["age".to_string()],
            directions: vec![OrderByDirection::Asc],
        };

        apply_order_by_from_precomputed(&mut to_order, precomputed, "default", &order_by_clause);

        assert_eq!(to_order, vec!["first", "second", "third", "fourth"]);
    }
    // TODO: add more tests
}
