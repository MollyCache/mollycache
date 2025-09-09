mod where_condition;
mod where_stack;
use crate::interpreter::ast::{WhereStackElement, WhereCondition};
use crate::db::table::{Table, Value};


// We create an interface here to allow us to create a spy for testing short circuiting.
trait MatchesWhereClause {
    fn matches_where_clause(&self, table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String>;
}

struct WhereConditionEvaluator;

impl MatchesWhereClause for WhereConditionEvaluator {
    fn matches_where_clause(&self, table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String> {
        where_condition::matches_where_clause(table, row, where_clause)
    }
}

// This is the public function that is used to check if a row matches a where stack.
pub fn row_matches_where_stack(table: &Table, row: &Vec<Value>, where_stack: &Vec<WhereStackElement>) -> Result<bool, String> {
    where_stack::matches_where_stack(table, row, where_stack, &WhereConditionEvaluator{})
}