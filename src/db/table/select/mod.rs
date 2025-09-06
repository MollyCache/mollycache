use crate::db::table::{Table, Value};
use crate::cli::ast::{SelectStatement};
use crate::db::table::helpers::{common::get_initial_rows, order_by_clause::get_ordered_rows, limit_clause::get_limited_rows};



pub fn select(table: &Table, statement: SelectStatement) -> Result<Vec<Vec<Value>>, String> {
    let mut rows = get_initial_rows(table, &statement)?;
    
    if let Some(order_by_clause) = statement.order_by_clause {
        rows = get_ordered_rows(table, rows, &order_by_clause)?;
    }

    if let Some(limit_clause) = &statement.limit_clause {
        rows = get_limited_rows(rows, limit_clause)?;
    }
    
    return Ok(rows);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition};
    use crate::cli::ast::{SelectStatementColumns, LimitClause, OrderByClause, OrderByDirection, Operator};
    use crate::cli::ast::WhereStackElement;
    use crate::cli::ast::WhereCondition;
    use crate::cli::ast::Operand;

    fn default_table() -> Table {
        Table {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {name: "id".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "name".to_string(), data_type: DataType::Text, constraints: vec![]},
                ColumnDefinition {name: "age".to_string(), data_type: DataType::Integer, constraints: vec![]},
                ColumnDefinition {name: "money".to_string(), data_type: DataType::Real, constraints: vec![]},
            ],
            rows: vec![
                vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
                vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
                vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
                vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
            ],
        }
    }

    #[test]
    fn select_with_all_tokens_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_specific_columns_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::Specific(vec!["name".to_string(), "age".to_string()]),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Text("John".to_string()), Value::Integer(25)],
            vec![Value::Text("Jane".to_string()), Value::Integer(30)],
            vec![Value::Text("Jim".to_string()), Value::Integer(35)],
            vec![Value::Null, Value::Integer(40)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("name".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Text("John".to_string())),
                }),
            ]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_selected_columns() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::Specific(vec!["name".to_string(), "age".to_string()]),
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("money".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Real(1000.0)),
                }),
            ]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Text("John".to_string()), Value::Integer(25)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_limit_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: Value::Integer(1),
                offset: Some(Value::Integer(1)),
            }),
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_with_where_clause_using_column_not_included_in_table_returns_error() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("column_not_included".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Text("John".to_string())),
                }),
            ]),
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Column column_not_included does not exist in table users");
    }

    #[test]
    fn select_with_order_by_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: Some(vec![OrderByClause {column: "money".to_string(), direction: OrderByDirection::Desc}]),
            limit_clause: None,
        };
        let result = select(&table, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }
}