use crate::cli::ast::{InsertIntoStatement, SelectStatementColumns};
pub mod select;

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

#[derive(Debug, PartialEq, PartialOrd)]
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

    pub fn clone(&self) -> Value {
        match self {
            Value::Integer(value) => Value::Integer(*value),
            Value::Real(value) => Value::Real(*value),
            Value::Text(value) => Value::Text(value.clone()),
            Value::Blob(value) => Value::Blob(value.clone()),
            Value::Null => Value::Null,
        }
    }
}

pub struct Table {
    pub _name: String,
    pub columns: Vec<ColumnDefinition>,
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    pub fn new(_name: String, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            _name,
            columns,
            rows: vec![],
        }
    }

    pub fn insert(&mut self, statement: InsertIntoStatement) -> Result<(), String> {
        // Validate columns
        if let Some(columns) = statement.columns {
            if columns.len() != self.columns.len() {
                return Err(format!("Columns have incorrect width"));
            }
            for (i, column) in columns.iter().enumerate() {
                if column != &self.columns[i].name {
                    return Err(format!("Column mismatch"));
                }
            }
        }
        
        let mut rows: Vec<Vec<Value>> = vec![];
        // Validate row inserts
        for row in statement.values {
            if row.len() != self.width() {
                return Err(format!("Rows have incorrect width"));
            }
            let row_values = self.validate_and_clone_row(&row)?;
            rows.push(row_values);
        }
        
        // Insert rows
        for row in rows {
            self.rows.push(row);
        }
        return Ok(());
    }

    



    pub fn get_column_from_row<'a>(&self, row: &'a Vec<Value>, column: &String) -> &'a Value {
        for (i, value) in row.iter().enumerate() {
            if self.columns[i].name == *column {
                return &value;
            }
        }
        return &Value::Null;
    }

    pub fn get_columns_from_row(&self, row: &Vec<Value>, selected_columns: &SelectStatementColumns) -> Result<Vec<Value>, String> {
        let mut row_values: Vec<Value> = vec![];
        if *selected_columns == SelectStatementColumns::All {
            return Ok(self.validate_and_clone_row(row)?);
        } else {
            let specific_selected_columns = selected_columns.columns()?;
            for (i, column) in self.columns.iter().enumerate() {
                if (*specific_selected_columns).contains(&column.name) {
                    row_values.push(row[i].clone());
                }
            }
        }
        return Ok(row_values);
    }

    fn width(&self) -> usize {
        self.columns.len()
    }

    fn validate_and_clone_row(&self, row: &Vec<Value>) -> Result<Vec<Value>, String> {
        if row.len() != self.width() {
            return Err(format!("Rows have incorrect width"));
        }

        let mut row_values: Vec<Value> = vec![];
        for (i, value) in row.iter().enumerate() {
            if value.get_type() != self.columns[i].data_type && value.get_type() != DataType::Null {
                return Err(format!("Data type mismatch for column {}", self.columns[i].name));
            }
            row_values.push(row[i].clone());
        }
        return Ok(row_values);
    }
}