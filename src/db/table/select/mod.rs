mod select_statement;
mod set_operator_evaluator;
use crate::db::{database::Database, table::Value};
use crate::interpreter::ast::{SelectStatementStack, SetOperator, SelectStatementStackElement};

pub fn select_statement_stack(database: &Database, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
    let mut evaluator = set_operator_evaluator::SetOperatorEvaluator::new();
    for element in statement.elements {
        match element {
            SelectStatementStackElement::SelectStatement(select_statement) => {
                let table = database.get_table(&select_statement.table_name)?;
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
    let result = evaluator.result()?;
    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::test_utils::default_database;
    use crate::interpreter::ast::{SelectStatement, SelectStatementColumns, WhereStackElement, WhereCondition, Operand, Operator, LogicalOperator};


    #[test]
    fn select_statement_stack_with_multiple_set_operators_works_correctly() {
        let database = default_database();
        let statement = SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
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
            elements: vec![
                SelectStatementStackElement::SelectStatement(SelectStatement {
                    table_name: "users".to_string(),
                    columns: SelectStatementColumns::All,
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
                    columns: SelectStatementColumns::All,
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
            elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
                where_clause: None,
                order_by_clause: None,
                limit_clause: None,
            }),
            SelectStatementStackElement::SelectStatement(SelectStatement {
                table_name: "users".to_string(),
                columns: SelectStatementColumns::All,
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
                columns: SelectStatementColumns::All,
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