use crate::cli::ast::{InsertIntoStatement, Operator, SelectStatement, SelectStatementColumns, WhereClause};

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
    _name: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Vec<Value>>,
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

    pub fn select(&self, statement: SelectStatement) -> Result<Vec<Vec<Value>>, String> {
        let mut rows: Vec<Vec<Value>> = vec![];
        if let Some(where_clause) = statement.where_clause {
            for row in self.rows.iter() {
                if self.matches_where_clause(&row, &where_clause) {
                    rows.push(self.get_columns_from_row(&row, &statement.columns)?);
                }
            }
        } else {
            todo!()
        }
        return Ok(rows);
    }

    fn matches_where_clause(&self, row: &Vec<Value>, where_clause: &WhereClause) -> bool {
        let column_value = self.get_column_from_row(row, &where_clause.column);
        if column_value.get_type() != where_clause.value.get_type() {
            return false;
        }

        match where_clause.operator {
            Operator::Equals => {
                return *column_value == where_clause.value;
            },
            Operator::NotEquals => {
                return *column_value != where_clause.value;
            },
            _ => {
                match column_value.get_type() {
                    DataType::Integer | DataType::Real => {
                        match where_clause.operator {
                            Operator::LessThan => {
                                return *column_value < where_clause.value;
                            },
                            Operator::GreaterThan => {
                                return *column_value > where_clause.value;
                            },
                            Operator::LessEquals => {
                                return *column_value <= where_clause.value;
                            },
                            Operator::GreaterEquals => {
                                return *column_value >= where_clause.value;
                            },
                            _ => {
                                return false;
                            },
                        }
                    },
                    _ => {
                        return false;
                    },
                }
            }
        }
    }

    fn get_column_from_row<'a>(&self, row: &'a Vec<Value>, column: &String) -> &'a Value {
        for (i, value) in row.iter().enumerate() {
            if self.columns[i].name == *column {
                return &value;
            }
        }
        return &Value::Null;
    }

    fn get_columns_from_row(&self, row: &Vec<Value>, columns: &SelectStatementColumns) -> Result<Vec<Value>, String> {
        let mut row_values: Vec<Value> = vec![];
        if *columns == SelectStatementColumns::All {
            return Ok(self.validate_and_clone_row(row)?);
        } else {
            for (i, column) in self.columns.iter().enumerate() {
                if self.columns.contains(column) {
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