use crate::db::table::Value;

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

    // Keeps duplicates
    pub fn union_all(&mut self) -> Result<(), String> {
        let mut first = self.pop()?;
        let second = self.pop()?;
        first.extend(second);
        self.push(first);
        Ok(())
    }
    
    pub fn union(&mut self) -> Result<(), String> {
        let mut first = self.pop()?;
        let second = self.pop()?;
        first.extend(second);
        self.push(first);
        Ok(())
    }
    pub fn intersect(&mut self) -> Result<(), String> {
        todo!()
    }
    pub fn except(&mut self) -> Result<(), String> {
        todo!()
    }

}