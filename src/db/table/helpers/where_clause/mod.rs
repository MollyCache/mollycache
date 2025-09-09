mod where_condition;
mod where_stack;
use crate::interpreter::ast::{WhereStackElement, WhereCondition};
use crate::db::table::{Table, Value};


// We create an interface here to allow us to create a spy for testing short circuiting.
trait MatchesWhereClause {
    fn matches_where_clause(&mut self, table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String>;
}

struct WhereConditionEvaluator;

impl MatchesWhereClause for WhereConditionEvaluator {
    fn matches_where_clause(&mut self, table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String> {
        where_condition::matches_where_clause(table, row, where_clause)
    }
}

// This is the public function that is used to check if a row matches a where stack.
pub fn row_matches_where_stack(table: &Table, row: &Vec<Value>, where_stack: &Vec<WhereStackElement>) -> Result<bool, String> {
    where_stack::matches_where_stack(table, row, where_stack, &mut WhereConditionEvaluator{})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, ColumnDefinition, DataType};
    use crate::interpreter::ast::{WhereStackElement, Operand, Operator, LogicalOperator};

    struct SpyWhereConditionEvaluator {
        conditions_evaluated: Vec<WhereCondition>,
    }

    impl MatchesWhereClause for SpyWhereConditionEvaluator {
        fn matches_where_clause(&mut self, table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String> {
            self.conditions_evaluated.push(where_clause.clone());
            where_condition::matches_where_clause(table, row, where_clause)
        }
    }


    #[test]
    fn matches_where_stack_works_short_circuits() {
        // WHERE (id = 1 OR id = 2);
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
        ]);
        let mut spy_where_condition_evaluator = SpyWhereConditionEvaluator{conditions_evaluated: vec![]};
        let row = vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ];
        let condition_1 = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))};
        let condition_2 = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(2))};
        let where_stack = vec![
            WhereStackElement::Condition(condition_1.clone()),
            WhereStackElement::Condition(condition_2.clone()),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ];
        let result = where_stack::matches_where_stack(&table, &row, &where_stack, &mut spy_where_condition_evaluator);
        assert!(result.is_ok() && result.unwrap());
        assert_eq!(spy_where_condition_evaluator.conditions_evaluated, vec![condition_1]);
    }

    #[test]
    fn matches_where_stack_does_not_short_circuit_if_condition_does_not_match() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
        ]);
        let mut spy_where_condition_evaluator = SpyWhereConditionEvaluator{conditions_evaluated: vec![]};
        let row = vec![
            Value::Integer(1),
            Value::Text("John".to_string()),
        ];
        let condition_1 = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(2))};
        let condition_2 = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))};
        let where_stack = vec![
            WhereStackElement::Condition(condition_1.clone()),
            WhereStackElement::Condition(condition_2.clone()),
            WhereStackElement::LogicalOperator(LogicalOperator::Or),
        ];
        let result = where_stack::matches_where_stack(&table, &row, &where_stack, &mut spy_where_condition_evaluator);
        assert!(result.is_ok() && result.unwrap());
        assert_eq!(spy_where_condition_evaluator.conditions_evaluated, vec![condition_1, condition_2]);
    }
}