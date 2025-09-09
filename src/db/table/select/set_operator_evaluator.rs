use std::collections::HashSet;

use crate::db::table::{helpers::common::remove_duplicate_rows, Value};

pub struct SetOperatorEvaluator {
    pub stack: Vec<Vec<Vec<Value>>>,
}

impl SetOperatorEvaluator {
    pub fn new() -> Self {
        Self {
            stack: vec![],
        }
    }

    pub fn result(&mut self) -> Result<Vec<Vec<Value>>, String> {
        if self.stack.len() != 1 {
            return Err("Error processing SELECT statement. Stack length is not 1".to_string());
        }
        self.pop()
    }

    pub fn push(&mut self, rows: Vec<Vec<Value>>) {
        self.stack.push(rows);
    }

    fn pop(&mut self) -> Result<Vec<Vec<Value>>, String> {
       self.stack.pop().ok_or("Error processing SELECT statement. Stack is empty".to_string())
    }

    pub fn union(&mut self) -> Result<(), String> {
        let second = self.pop()?;
        let mut first = self.pop()?;
        first.extend(second.into_iter());
        let result = remove_duplicate_rows(first);
        self.push(result);
        Ok(())
    }
    
    pub fn union_all(&mut self) -> Result<(), String> {
        let second = self.pop()?;
        let mut first = self.pop()?;
        first.extend(second);
        self.push(first);
        Ok(())
    }

    pub fn intersect(&mut self) -> Result<(), String> {
        let second = self.pop()?.into_iter().collect::<HashSet<Vec<Value>>>();
        let mut first = self.pop()?;
        let mut index: usize = 0;
        while index < first.len() {
            if second.contains(&first[index]) {
                index += 1;
            }
            else {
                first.swap_remove(index);
            }
        }
        self.push(first);
        Ok(())
    }

    pub fn except(&mut self) -> Result<(), String> {
        let second = self.pop()?.into_iter().collect::<HashSet<Vec<Value>>>();
        let mut first = self.pop()?;
        let mut index: usize = 0;
        while index < first.len() {
            if second.contains(&first[index]) {
                first.swap_remove(index);
            }
            else {
                index += 1;
            }
        }
        self.push(first);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::table::test_utils::assert_table_rows_eq_unordered;

    fn rows_1() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
        ]
    }

    fn rows_2() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1), Value::Text("Fletcher".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Null, Value::Real(5000.0)],
        ]
    }

    #[test]
    fn union_all_works_correctly() {
        let mut evaluator = SetOperatorEvaluator::new();
        evaluator.push(rows_1());
        evaluator.push(rows_2());
        assert!(evaluator.union_all().is_ok());
        let result = evaluator.result();
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
            vec![Value::Integer(1), Value::Text("Fletcher".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Null, Value::Real(5000.0)],
        ];
        assert!(result.is_ok());
        assert_table_rows_eq_unordered(expected, result.unwrap());
    }

    #[test]
    fn intersect_works_correctly() {
        let mut evaluator = SetOperatorEvaluator::new();
        evaluator.push(rows_1());
        evaluator.push(rows_2());
        assert!(evaluator.intersect().is_ok());
        let result = evaluator.result();
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(2), Value::Text("Jane".to_string()), Value::Integer(30), Value::Real(2000.0)],
        ];
        assert_table_rows_eq_unordered(expected, result.unwrap());
    }

    #[test]
    fn except_works_correctly() {
        let mut evaluator = SetOperatorEvaluator::new();
        evaluator.push(rows_1());
        evaluator.push(rows_2());
        assert!(evaluator.except().is_ok());
        let result = evaluator.result();
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1), Value::Text("John".to_string()), Value::Integer(25), Value::Real(1000.0)],
            vec![Value::Integer(3), Value::Text("Jim".to_string()), Value::Integer(35), Value::Real(3000.0)],
        ];
        assert_table_rows_eq_unordered(expected, result.unwrap());
    }
}
    
    
