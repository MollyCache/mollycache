use crate::db::table::core::value::Value;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
#[repr(transparent)]
pub struct Row(pub Vec<Value>);

#[derive(Debug)]
pub struct RowStack {
    pub stack: Vec<Row>,
}

impl Deref for Row {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RowStack {
    pub fn new(stack: Row) -> Self {
        Self { stack: vec![stack] }
    }
}
