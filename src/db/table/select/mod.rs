mod select_statement;
mod set_operator_evaluator;
use crate::db::{database::Database, table::Value};
use crate::interpreter::ast::{SelectStatementStack, SetOperator, SelectStatementStackElement, SelectableStack, SelectableStackElement};
use crate::db::table::helpers::{order_by_clause::{perform_comparions}, limit_clause::get_limited_rows};


pub fn select_statement_stack(database: &Database, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
    // TODO: this
    /*
    let mut evaluator = set_operator_evaluator::SetOperatorEvaluator::new();
    let statement_columns = statement.columns.columns();
    let mut columns: Option<Vec<&String>> = match statement_columns {
        Err(_) => None,
        Ok(columns_list) => Some(columns_list),
    };
    for element in statement.elements {
        match element {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                let table = database.get_table(&select_statement.table_name)?;
                columns = match columns {
                    None => Some(table.get_columns()),
                    Some(columns) => {
                        if statement.columns == SelectStatementColumns::All {
                            if table.get_columns() != columns {
                                return Err(format!("Columns mismatch between SELECT statements in Union"));
                            }
                        }
                        else {
                            if statement.columns.columns()? != columns {
                                return Err(format!("Columns mismatch between SELECT statements in Union"));
                            }
                        }
                        Some(columns)
                    },
                };
                let rows = select_statement::select_statement(table, &select_statement)?;
                evaluator.push(rows);
            }
            SelectStatementStackElement::SetOperator(set_operator) => {
                match set_operator {
                    SetOperator::UnionAll => {
                        evaluator.union_all()?;
                    }
                    SetOperator::Union => {
                        evaluator.union()?;
                    }
                    SetOperator::Intersect => {
                        evaluator.intersect()?;
                    }
                    SetOperator::Except => {
                        evaluator.except()?;
                    }
                }
            }
        }
    }
    let mut result = evaluator.result()?;
    if let Some(order_by_clause) = statement.order_by_clause {
        result.sort_by(|a, b| {
            if let Some(columns) = &columns {
                perform_comparions(&columns, a, b, &order_by_clause)
            }
            else {
                unreachable!()
            }
        });
    }
    if let Some(limit_clause) = statement.limit_clause {
        result = get_limited_rows(result, &limit_clause)?;
    }
    Ok(result)
    */
    Ok(vec![])
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::test_utils::default_database;
    use crate::interpreter::ast::{SelectStatement, SelectableStack, SelectableStackElement, WhereStackElement, WhereCondition, Operand, Operator, LogicalOperator};


    #[test]
    fn select_statement_stack_with_multiple_set_operators_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            })],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
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
    fn select_statement_stack_with_set_operator_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All]
                    },
                    where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                        l_side: Operand::Identifier("id".to_string()),
                        operator: Operator::Equals,
                        r_side: Operand::Value(Value::Integer(1)),
                    })]),
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All]
                    },
                    where_clause: None,
                    order_by_clause: None,
                    limit_clause: None,
                }),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            ],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_statement_stack_works_correctly_with_multiple_set_operators() {
        let database = default_database();
        let statement = SelectStatementStack {
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All]
            },
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }),
            SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                }),
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(2)),
                }),
                WhereStackElement::LogicalOperator(LogicalOperator::Or),
                ]),
                order_by_clause: None,
                limit_clause: None,
            }),
            SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                })]),
                order_by_clause: None,
                limit_clause: None,
            }),
            SelectStatementStackElement::SetOperator(SetOperator::Except),
            ],
            order_by_clause: None,
            limit_clause: None,
        };
        let result = select_statement_stack(&database, statement);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
        ];
        assert_eq!(expected, result.unwrap());
    }
}