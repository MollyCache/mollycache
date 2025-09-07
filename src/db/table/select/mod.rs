mod select_statement;
mod set_operator_evaluator;
use crate::db::{database::Database, table::Value};
use crate::interpreter::ast::{SelectStatementStack, SetOperator, SelectStatementStackElement};

pub fn select_statement_stack(database: &Database, statement: SelectStatementStack) -> Result<Vec<Vec<Value>>, String> {
    let mut evaluator = set_operator_evaluator::SetOperatorEvaluator {
        stack: vec![],
    };
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
                        evaluator.union_all();
                    }
                    SetOperator::Union => {
                        evaluator.union();
                    }
                    SetOperator::Intersect => {
                        evaluator.intersect();
                    }
                    SetOperator::Except => {
                        evaluator.except();
                    }
                }
            }
        }
    }
    let result = evaluator.result()?;
    Ok(result)
}

