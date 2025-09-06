use std::collections::HashSet;

use crate::db::table::Table;
use crate::cli::ast::DeleteStatement;
use crate::db::table::helpers::{
    common::get_row_indicies_matching_where_clause, 
    order_by_clause::get_ordered_row_indicies, 
    limit_clause::get_limited_row_indicies
};


pub fn delete(table: &mut Table, statement: DeleteStatement) -> Result<(), String> {
    let row_indicies_to_delete = get_row_indicies_to_delete(table, statement)?;
    swap_remove_bulk(table, row_indicies_to_delete)?;
    Ok(())
}


fn get_row_indicies_to_delete(table: &mut Table, statement: DeleteStatement) -> Result<Vec<usize>, String> {
    let mut row_indicies = get_row_indicies_matching_where_clause(table, statement.where_clause)?;

    if let Some(order_by_clause) = statement.order_by_clause {
        row_indicies = get_ordered_row_indicies(table, row_indicies, &order_by_clause)?;
    }

    if let Some(limit_clause) = statement.limit_clause {
        row_indicies = get_limited_row_indicies(row_indicies, &limit_clause)?;
    }

    return Ok(row_indicies);
}

fn swap_remove_bulk(table: &mut Table, row_indicies: Vec<usize>) -> Result<(), String> {
    let table_len = table.rows.len()-1;
    let mut row_indicies_set = row_indicies.iter().collect::<HashSet<&usize>>();
    let mut right_pointer = 0;
    let mut iter = row_indicies.iter();
    
    while let Some(to_swap) = iter.next() {
        if *to_swap == (table_len - right_pointer) {
            row_indicies_set.remove(to_swap);
            right_pointer += 1;
        }
        else {
            table.rows.swap(*to_swap, table_len - right_pointer);
            row_indicies_set.remove(to_swap);
            right_pointer += 1;
        }
    }
    for _ in 0..right_pointer {
        table.rows.pop();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::Value;
    use crate::cli::ast::{WhereStackElement, Operator, Operand, WhereCondition, OrderByDirection, OrderByClause};
    use crate::db::table::test_utils::{default_table, assert_table_rows_eq_unordered};
    use crate::cli::ast::LimitClause;

    #[test]
    fn delete_from_table_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(2)),
                })
            ]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
        ];
        assert_table_rows_eq_unordered(expected, table.rows);
    }

    #[test]
    fn delete_from_table_with_all_clauses_works_correctly() {
        let mut table = default_table();
        table.rows = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
            vec![Value::Integer(5), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(6), Value::Text("John".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(7), Value::Text("John".to_string()), Value::Integer(35), Value::Real(3000.0)],
        ];
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition { l_side: Operand::Identifier("name".to_string()), operator: Operator::Equals, r_side: Operand::Value(Value::Text("John".to_string())) })]),
            order_by_clause: Some(vec![OrderByClause { column: "id".to_string(), direction: OrderByDirection::Desc }]),
            limit_clause: Some(LimitClause { limit: Value::Integer(1), offset: Some(Value::Integer(2)) }),
        };
        let result = delete(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
            vec![Value::Integer(6), Value::Text("John".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(7), Value::Text("John".to_string()), Value::Integer(35), Value::Real(3000.0)],
        ];
        assert_table_rows_eq_unordered(expected, table.rows);
    }

    #[test]
    fn delete_multiple_rows_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition { l_side: Operand::Identifier("id".to_string()), operator: Operator::GreaterThan, r_side: Operand::Value(Value::Integer(1)) })]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
        ];
        assert_table_rows_eq_unordered(expected, table.rows);
    }

    #[test]
    fn delete_all_rows_works_correctly() {
        let mut table = default_table();
        let statement = DeleteStatement {
            table_name: "users".to_string(),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = delete(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![];
        assert_table_rows_eq_unordered(expected, table.rows);
    }
}