use crate::db::table::Table;
use crate::interpreter::ast::{UpdateStatement, ColumnValue};
use crate::db::table::helpers::common::get_row_indicies_matching_clauses;
use crate::db::table::DataType;

pub fn update(table: &mut Table, statement: UpdateStatement) -> Result<Vec<usize>, String> {
    let row_indicies = get_row_indicies_matching_clauses(table, &statement.where_clause, &statement.order_by_clause, &statement.limit_clause)?;
    update_rows_from_indicies(table, &row_indicies, statement.update_values)?;
    Ok(row_indicies)
}

fn update_rows_from_indicies(table: &mut Table, row_indicies: &Vec<usize>, update_values: Vec<ColumnValue>) -> Result<(), String> {
    for row_index in row_indicies {
        for update_value in &update_values {
            let column_index = table.get_index_of_column(&update_value.column)?;
            if table.columns[column_index].data_type != update_value.value.get_type() && update_value.value.get_type() != DataType::Null {
                return Err(format!("Found different data types for column: {} and value: {:?}", update_value.column, update_value.value.get_type()));
            }
            table[*row_index][column_index] = update_value.value.clone();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Value, DataType, ColumnDefinition};
    use crate::interpreter::ast::ColumnValue;
    use crate::db::table::test_utils::{default_table, assert_table_rows_eq_unordered};
    use crate::interpreter::ast::{WhereStackElement, WhereCondition, Operand, Operator, OrderByClause, OrderByDirection, LimitClause, SelectableStack, SelectableStackElement};
    use crate::db::table::Row;

    #[test]
    fn update_works_correctly() {
        let mut table = default_table();
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "name".to_string(), value: Value::Text("John".to_string()) }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("John".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("John".to_string()), Value::Integer(35), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Text("John".to_string()), Value::Integer(40), Value::Real(4000.0)]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn update_with_all_clauses_works_correctly() {
        let mut table = default_table();
        table.set_rows(vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)]),
            Row(vec![Value::Integer(5), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(6), Value::Text("John".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(7), Value::Text("John".to_string()), Value::Integer(35), Value::Real(3000.0)]),
        ]);
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "name".to_string(), value: Value::Text("Fletcher".to_string()) }],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition { l_side: Operand::Identifier("name".to_string()), operator: Operator::Equals, r_side: Operand::Value(Value::Text("John".to_string())) })]),
            order_by_clause: Some(OrderByClause {
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                },
                directions: vec![OrderByDirection::Desc],
            }),
            limit_clause: Some(LimitClause { limit: 1, offset: Some(2) }),
        };
        let result = update(&mut table, statement);
        assert!(result.is_ok());
        let expected = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)]),
            Row(vec![Value::Integer(5), Value::Text("Fletcher".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(6), Value::Text("John".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(7), Value::Text("John".to_string()), Value::Integer(35), Value::Real(3000.0)]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn update_multiple_columns_and_rows_works_correctly() {
        let mut table = default_table();
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "name".to_string(), value: Value::Text("Fletcher".to_string()) }, ColumnValue { column: "age".to_string(), value: Value::Integer(50) }],
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition { l_side: Operand::Identifier("id".to_string()), operator: Operator::GreaterThan, r_side: Operand::Value(Value::Integer(1)) })]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_ok());
        let row_indicies = result.unwrap();
        assert_eq!(vec![1, 2, 3], row_indicies);
        let expected = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Fletcher".to_string()), Value::Integer(50), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("Fletcher".to_string()), Value::Integer(50), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Text("Fletcher".to_string()), Value::Integer(50), Value::Real(4000.0)]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn update_empty_table_works_correctly() {
        let mut table = Table::new(
            "users".to_string(), 
            vec![ColumnDefinition { name: "id".to_string(), data_type: DataType::Integer, constraints: vec![] }],
        );
        table.rows = vec![];
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "name".to_string(), value: Value::Text("Fletcher".to_string()) }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
        let expected = vec![];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }

    #[test]
    fn update_with_invalid_column_works_correctly() {
        let mut table = default_table();
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "invalid".to_string(), value: Value::Text("Fletcher".to_string()) }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Column invalid does not exist in table users");
    }

    #[test]
    fn update_with_invalid_value_works_correctly() {
        let mut table = default_table();
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "name".to_string(), value: Value::Integer(1) }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Found different data types for column: name and value: Integer");
    }

    #[test]
    fn update_with_null_value_works_correctly() {
        let mut table = default_table();
        let statement = UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue { column: "money".to_string(), value: Value::Null }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = update(&mut table, statement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0, 1, 2, 3]);
        let expected = vec![
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Null]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Null]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Null]),
            Row(vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Null]),
        ];
        assert_table_rows_eq_unordered(expected, table.get_rows_clone());
    }
}