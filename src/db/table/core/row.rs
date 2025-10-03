use crate::db::table::core::value::Value;
use std::ops::{Deref, DerefMut};
use std::cmp::Ordering;

#[derive(Debug, Hash, Clone)]
#[repr(transparent)]
pub struct Row(pub Vec<Value>);

#[derive(Debug, Clone)]
pub struct RowStack {
    pub stack: Vec<Row>,
}

impl Row {
    pub fn exactly_equal(self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (first, second) in self.iter().zip(other.iter()) {
            if !first.exactly_equal(second) {
                return false;
            }
        }
        true
    }
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

impl PartialOrd for Row {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.len() < other.len() {
            return Some(Ordering::Less);
        } else if self.len() > other.len() {
            return Some(Ordering::Greater);
        }
        for (first, second) in self.iter().zip(other.iter()) {
            let ordering = first.partial_cmp(second);
            if ordering == None {
                return None;
            } else if ordering != Some(Ordering::Equal) {
                return ordering;
            }
        }
        Some(Ordering::Equal)
    }
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Eq for Row {}

impl RowStack {
    pub fn new(stack: Row) -> Self {
        Self { stack: vec![stack] }
    }

    pub fn new_with_stack(stack: Vec<Row>) -> Self {
        Self { stack }
    }

    pub fn append_clone(&mut self) {
        self.stack.push(self.stack.last().unwrap().clone());
    }

    pub fn exactly_equal(&self, other: &Self) -> bool {
        if self.stack.len() != other.stack.len() {
            return false;
        }
        for (first, second) in self.stack.iter().zip(other.stack.iter()) {
            if !first.clone().exactly_equal(second) {
                return false;
            }
        }
        true
    }
}
