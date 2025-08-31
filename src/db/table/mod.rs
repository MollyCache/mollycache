pub mod select;
pub mod insert;
pub mod common;

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

    fn width(&self) -> usize {
        self.columns.len()
    }
}