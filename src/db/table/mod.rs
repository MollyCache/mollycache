use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::interpreter::ast::OrderByDirection;

pub mod select;
pub mod insert;
pub mod delete;
pub mod update;
pub mod create_table;
pub mod helpers;
pub mod drop_table;
pub mod alter_table;
#[cfg(test)]
pub mod test_utils;


#[derive(Debug, PartialEq)]
pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
    Null,
}

#[derive(Debug, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub constraints: Vec<ColumnConstraint>,
}

#[derive(Debug, PartialEq)]
pub struct ColumnConstraint {
    pub constraint_type: String,
}

#[derive(Debug, PartialOrd, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Null
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Value::Integer(_) => DataType::Integer,
            Value::Real(_) => DataType::Real,
            Value::Text(_) => DataType::Text,
            Value::Blob(_) => DataType::Blob,
            Value::Null => DataType::Null,
        }
    }

    pub fn compare(&self, other: &Value, direction: &OrderByDirection) -> Ordering {
        let result = match (self, other) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Real(a), Value::Real(b)) => {
                if a.is_nan() && b.is_nan() {
                    Ordering::Equal
                } else if a.is_nan() {
                    Ordering::Less
                } else if b.is_nan() {
                    Ordering::Greater
                } else {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                }
            },
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            _ => return Ordering::Equal, // Bad - returns equal if data types are different
        };

        if direction == &OrderByDirection::Desc {
            result.reverse()
        } else {
            result
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Real(f) => Some(*f as i64),
            _ => None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Integer(i) => Some(*i as f64),
            Value::Real(f) => Some(*f),
            _ => None
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Real(a), Value::Real(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            },
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Blob(a), Value::Blob(b)) => a == b,
            (Value::Null, Value::Null) => true, // TODO: Bad - NULL == NULL should be false but this breaks assert_eq!
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Integer(i) => {
                0u8.hash(state);
                i.hash(state);
            }
            Value::Real(f) => {
                1u8.hash(state);
                if f.is_nan() {
                    u64::MAX.hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::Text(s) => {
                2u8.hash(state);
                s.hash(state);
            }
            Value::Blob(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            Value::Null => {
                4u8.hash(state);
            }
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    pub fn new(name: String, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            name,
            columns,
            rows: vec![],
        }
    }

    pub fn get_column_from_row<'a>(&self, row: &'a Vec<Value>, column: &String) -> &'a Value {
        for (i, value) in row.iter().enumerate() {
            if self.columns[i].name == *column {
                return &value;
            }
        }
        return &Value::Null;
    }

    pub fn has_column(&self, column: &String) -> bool {
        self.columns.iter().any(|c| c.name == *column)
    }

    fn width(&self) -> usize {
        self.columns.len()
    }

    pub fn get_index_of_column(&self, column: &String) -> Result<usize, String> {
        for (i, c) in self.columns.iter().enumerate() {
            if c.name == *column {
                return Ok(i);
            }
        }
        return Err(format!("Column {} does not exist in table {}", column, self.name));
    }

    pub fn get_columns(&self) -> Vec<&String> {
        self.columns.iter().map(|column| &column.name).collect()
    }
}