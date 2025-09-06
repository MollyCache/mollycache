use std::cmp::Ordering;

use crate::cli::ast::OrderByDirection;

pub mod select;
pub mod insert;
pub mod delete;
pub mod helpers;
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

#[derive(Debug, PartialEq, PartialOrd, Clone)]
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
                if a > b {
                    Ordering::Greater
                } else if a < b {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            
            },
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            _ => return Ordering::Equal, // Bad - returns equal if data types are different
        };

        if direction == &OrderByDirection::Asc {
            return result;
        } else {
            return result.reverse();
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
}