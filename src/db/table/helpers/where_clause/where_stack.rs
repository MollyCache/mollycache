use crate::db::table::{Table, Row};
use crate::interpreter::ast::{WhereStackElement, LogicalOperator, WhereCondition};
use crate::db::table::helpers::where_clause::MatchesWhereClause;


// This file holds the logic for whether a row matches a where stack which is a vec of WhereConditions
// and logical operators stored in Reverse Polish Notation.

enum Condition<'a> {
    Boolean(bool),
    WhereCondition(&'a WhereCondition),
}

impl<'a> Condition<'a> {
    fn evaluate(&self, table: &Table, row: &Row, where_clause_evaluator: &mut dyn MatchesWhereClause) -> Result<bool, String> {
        match self {
            Condition::Boolean(boolean) => Ok(*boolean),
            Condition::WhereCondition(where_condition) => where_clause_evaluator.matches_where_clause(table, row, where_condition),
        }
    }
}

pub fn matches_where_stack(table: &Table, row: &Row, where_stack: &Vec<WhereStackElement>, where_clause_evaluator: &mut dyn MatchesWhereClause) -> Result<bool, String> {
    let mut result_stack: Vec<Condition> = vec![];
    for where_stack_element in where_stack {
        match where_stack_element {
            WhereStackElement::Condition(where_condition) => {
                result_stack.push(Condition::WhereCondition(where_condition));
            },
            WhereStackElement::LogicalOperator(logical_operator) => {
                let pop1 = match result_stack.pop() {
                    Some(pop1) => pop1,
                    None => return Err(format!("Error evaluating where clause with table: {:?}", table)),
                };
                match logical_operator {
                    LogicalOperator::Not => {      
                        result_stack.push(Condition::Boolean(!pop1.evaluate(table, row, where_clause_evaluator)?));
                    }
                    LogicalOperator::And => {
                        let pop2 = match result_stack.pop() {
                            Some(pop2) => pop2,
                            None => return Err(format!("Error evaluating where clause with table: {:?}", table)),
                        };
                        result_stack.push(Condition::Boolean(pop2.evaluate(table, row, where_clause_evaluator)? && pop1.evaluate(table, row, where_clause_evaluator)?));
                    }
                    LogicalOperator::Or => {
                        let pop2 = match result_stack.pop() {
                            Some(pop2) => pop2,
                            None => return Err(format!("Error evaluating where clause with table: {:?}", table)),
                        };
                        result_stack.push(Condition::Boolean(pop2.evaluate(table, row, where_clause_evaluator)? || pop1.evaluate(table, row, where_clause_evaluator)?));
                    }
                }
            },
            _ => unreachable!(), // There should be no Parentheses in the final where stack.
        }
    }
    
    if let Some(result) = result_stack.pop() {
        return Ok(result.evaluate(table, row, where_clause_evaluator)?);
    }
    else {
        return Err(format!("Error evaluating where clause with table: {:?}", table));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, ColumnDefinition, DataType, Row};
    use crate::interpreter::ast::{WhereStackElement, LogicalOperator};
    use crate::interpreter::ast::{Operator, Operand, WhereCondition};
    use crate::db::table::helpers::where_clause::WhereConditionEvaluator;

    fn simple_condition(l_side: &str, operator: Operator, r_side: Value) -> WhereStackElement {
        WhereStackElement::Condition(WhereCondition {l_side: Operand::Identifier(l_side.to_string()), operator, r_side: Operand::Value(r_side)})
    }

    #[test]
    fn matches_where_stack_returns_true_if_row_matches_where_stack_with_single_condition() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = Row(vec![Value::Integer(1)]);
        let where_stack = vec![WhereStackElement::Condition(WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))})];
        let result = matches_where_stack(&table, &row, &where_stack, &mut WhereConditionEvaluator{});
        assert!(result.is_ok() && result.unwrap());
    }

    #[test]
    fn matches_where_stack_works_with_complex_conditions() {
        // WHERE (id = 1 OR NOT (name = "John" AND age > 20));
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
            ColumnDefinition {name:"age".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = Row(vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
            Value::Integer(20),
        ]);
        let where_stack = vec![
            simple_condition("id", Operator::Equals, Value::Integer(1)),
            simple_condition("name", Operator::Equals, Value::Text("John".to_string())),
            simple_condition("age", Operator::GreaterThan, Value::Integer(20)),
            WhereStackElement::LogicalOperator(LogicalOperator::And),
            WhereStackElement::LogicalOperator(LogicalOperator::Not),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ];
        let result = matches_where_stack(&table, &row, &where_stack, &mut WhereConditionEvaluator{});
        assert!(result.is_ok() && result.unwrap());

        let row = Row(vec![
            Value::Integer(2),
            Value::Text("Fletcher".to_string()),
            Value::Integer(15),
        ]);
        let result = matches_where_stack(&table, &row, &where_stack, &mut WhereConditionEvaluator{});
        assert!(result.is_ok() && result.unwrap());

        let row = Row(vec![
            Value::Integer(2),
            Value::Text("John".to_string()),
            Value::Integer(25),
        ]);
        let result = matches_where_stack(&table, &row, &where_stack, &mut WhereConditionEvaluator{});
        assert!(result.is_ok() && !result.unwrap());
    }
}