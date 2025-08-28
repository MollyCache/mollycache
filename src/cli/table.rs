pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean
}

pub struct Table {
    name: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Row>,
    length: usize
}

pub struct ColumnDefinition {
    name: String,
    data_type: DataType
}

struct Row {
    primary_key: usize,
    values: Vec<Value>
}

pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Bool(bool)
}