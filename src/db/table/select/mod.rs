mod select_statement;
mod set_operator_evaluator;
use crate::db::{database::Database, table::Row};
use crate::interpreter::ast::{SelectStatementStack, SetOperator, SelectStatementStackElement};
use crate::db::table::helpers::order_by_clause::apply_order_by;


pub fn select_statement_stack(database: &Database, statement: SelectStatementStack) -> Result<Vec<Row>, String> {
    let mut evaluator = set_operator_evaluator::SetOperatorEvaluator::new();
    let mut column_names: Option<Vec<String>> = None;
    
    // TODO: so ugly and also just false. Needed in some sort of way for now. See later TODO about dealing with 2+ tables
    let mut first_table = None;

    for element in statement.elements {
        match element {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                match &column_names {
                    Some(column_names) => {
                        if select_statement.column_names.len() != column_names.len() {
                            return Err(format!("Parse error: SELECTs to the left and right of UNION do not have the same number of result columns"));
                        }
                    },
                    None => {
                        column_names = Some(select_statement.column_names.clone())
                    }
                }

                let table = database.get_table(&select_statement.table_name)?;
                let rows = select_statement::select_statement(table, &select_statement)?;
                evaluator.push(rows);

                if first_table.is_none() {
                    first_table = Some(table);
                }
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
        if let Some(table) = first_table {
            // TODO: this is just plain false when working with 2+ tables
            apply_order_by(table, &mut result, &order_by_clause)?;
        } else {
            unreachable!();
        }
    }

    // TODO: if LIMIT without ORDER BY, apply LIMIT at the beginning / after the WHERE
    if let Some(limit_clause) = statement.limit_clause {
        let offset = limit_clause.offset.unwrap_or(0);
        let end = limit_clause.limit + offset;
        result = result[offset..end].to_vec();
    }
    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::test_utils::default_database;
    use crate::db::table::Value;
    use crate::interpreter::ast::{SelectStatement, SelectableStack, SelectableStackElement, WhereStackElement, WhereCondition, Operand, Operator, LogicalOperator, SelectMode};


    #[test]
    fn select_statement_stack_with_multiple_set_operators_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                column_names: vec!["*".to_string()],
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
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
            Row(vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)]),
            Row(vec![Value::Integer(4), Value::Null, Value::Integer(40), Value::Real(4000.0)]),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_statement_stack_with_set_operator_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    mode: SelectMode::All,
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All]
                    },
                    column_names: vec!["*".to_string()],
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
                    mode: SelectMode::All,
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All]
                    },
                    column_names: vec!["*".to_string()],
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
            Row(vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)]),
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn select_statement_stack_works_correctly_with_multiple_set_operators() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                column_names: vec!["*".to_string()],
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }),
            SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                column_names: vec!["*".to_string()],
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
                mode: SelectMode::All,
                columns: SelectableStack {
                    selectables: vec![SelectableStackElement::All]
                },
                column_names: vec!["*".to_string()],
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
            Row(vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)]),
        ];
        assert_eq!(expected, result.unwrap());
    }
}