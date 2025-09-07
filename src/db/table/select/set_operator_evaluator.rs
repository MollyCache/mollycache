use crate::db::table::Value;

pub struct SetOperatorEvaluator {
    pub stack: Vec<Vec<Vec<Value>>>,
}

impl SetOperatorEvaluator {
    pub fn result(&mut self) -> Result<Vec<Vec<Value>>, String> {
        self.stack.pop().ok_or()
    }

    pub fn union_all(&mut self) {
        todo!()
    }

    pub fn union(&mut self) {
        todo!()
    }
    pub fn intersect(&mut self) {
        todo!()
    }
    pub fn except(&mut self) {
        todo!()
    }
    pub fn push(&mut self, rows: Vec<Vec<Value>>) {
        self.stack.push(rows);
    }
}