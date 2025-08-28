pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
    Null,
}

pub struct Table {
    name: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Row>,
    length: usize,
}

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub constraints: Vec<ColumnConstraint>,
}

pub struct ColumnConstraint {
    pub constraint_type: String,
}

struct Row {
    primary_key: usize,
    values: Vec<Value>,
}

pub enum Value {
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Null
}
