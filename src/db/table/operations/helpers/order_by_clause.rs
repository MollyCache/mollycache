use std::cmp::Ordering;

use crate::db::table::core::row::Row;
use crate::interpreter::ast::{OrderByClause, OrderByDirection};

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
    for (i, direction) in order_by_clause.directions.iter().enumerate() {
        let ordering = row1[i].partial_cmp(&row2[i]).unwrap_or(Ordering::Equal);
        if ordering != Ordering::Equal {
            return if *direction == OrderByDirection::Desc {
                ordering.reverse()
            } else {
                ordering
            };
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
    use crate::interpreter::ast::SelectableColumn;
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
            columns: vec![SelectableColumn {
                selectables: vec![SelectableStackElement::Column("age".to_string())],
                column_name: "age".to_string(),
            }],
            directions: vec![OrderByDirection::Asc],
        };

        apply_order_by_from_precomputed(&mut to_order, precomputed, "default", &order_by_clause);

        assert_eq!(to_order, vec!["first", "second", "third", "fourth"]);
    }

    #[test]
    fn apply_order_by_from_precomputed_multiple_columns_desc() {
        let mut to_order = vec!["fifth", "second", "sixth", "third", "fourth", "first"];

        let precomputed = vec![
            Row(vec![Value::Integer(10), Value::Real(1.0)]),
            Row(vec![Value::Integer(35), Value::Real(2.0)]),
            Row(vec![Value::Integer(3), Value::Real(-10.5)]),
            Row(vec![Value::Integer(35), Value::Real(3.0)]),
            Row(vec![Value::Integer(10), Value::Real(0.0)]),
            Row(vec![Value::Integer(35), Value::Real(-2.5)]),
        ];

        let order_by_clause = OrderByClause {
            columns: vec![
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("age".to_string())],
                    column_name: "age".to_string(),
                },
                SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("money".to_string())],
                    column_name: "money".to_string(),
                },
            ],
            directions: vec![OrderByDirection::Desc, OrderByDirection::Asc],
        };

        apply_order_by_from_precomputed(&mut to_order, precomputed, "default", &order_by_clause);

        assert_eq!(
            to_order,
            vec!["first", "second", "third", "fourth", "fifth", "sixth"]
        );
    }
}
