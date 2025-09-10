use crate::db::table::{Table, Value};
use crate::interpreter::ast::{SelectStatement};
use crate::db::table::helpers::common::{get_row_indicies_matching_clauses, get_row_columns_from_indicies};



pub fn select_statement(table: &Table, statement: &SelectStatement) -> Result<Vec<Vec<Value>>, String> {
    let row_indicies = get_row_indicies_matching_clauses(table, &statement.where_clause, &statement.order_by_clause, &statement.limit_clause)?;
    
    return Ok(get_row_columns_from_indicies(table, row_indicies, Some(&statement.columns))?);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::Value;
    use crate::interpreter::ast::{SelectableStack, SelectableStackElement, LimitClause, OrderByClause, OrderByDirection, Operator};
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::Operand;
    use crate::db::table::test_utils::default_table;

    #[test]
    fn select_with_all_tokens_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            column_names: vec!["*".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
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
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("name".to_string()),
                    SelectableStackElement::Column("age".to_string()),
                ]
            },
            column_names: vec!["name".to_string(), "age".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            column_names: vec!["*".to_string()],
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
        let result = select_statement(&table, &statement);
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
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("name".to_string()),
                    SelectableStackElement::Column("age".to_string()),
                ]
            },
            column_names: vec!["name".to_string(), "age".to_string()],
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
        let result = select_statement(&table, &statement);
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            column_names: vec!["*".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: Value::Integer(1),
                offset: Some(Value::Integer(1)),
            }),
        };
        let result = select_statement(&table, &statement);
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            column_names: vec!["*".to_string()],
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
        let result = select_statement(&table, &statement);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Column column_not_included does not exist in table users");
    }

    #[test]
    fn select_with_order_by_clause_is_generated_correctly() {
        let table = default_table();
        let statement = SelectStatement {
            table_name: "users".to_string(),
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            column_names: vec!["*".to_string()],
            where_clause: None,
            order_by_clause: Some(vec![OrderByClause {column: "money".to_string(), direction: OrderByDirection::Desc}]),
            limit_clause: None,
        };
        let result = select_statement(&table, &statement);
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