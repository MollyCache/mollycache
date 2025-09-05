use crate::cli::ast::{Operator, WhereCondition, Operand, WhereStackElement};
use crate::db::table::{Table, Value, DataType};

// For now this function only supports one column = value where clause

pub fn matches_where_stack(table: &Table, row: &Vec<Value>, where_stack: &Vec<WhereStackElement>) -> Result<bool, String> {
    let where_condition = match where_stack.first() {
        Some(WhereStackElement::Condition(where_condition)) => where_condition,
        _ => return Err(format!("Found nothing when expected edge")),
    };
    
    if let Operand::Identifier(column_name) = &where_condition.l_side {
        if !table.has_column(column_name) {
            return Err(format!("Column {} does not exist in table {}", column_name, table.name));
        }
    }
    
    matches_where_clause(table, row, where_condition)
}

pub fn matches_where_clause(table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String> {
    let l_side = match &where_clause.l_side {
        Operand::Identifier(column) => column,
        _ => return Err(format!("Found invalid left side of condition: {:?}", where_clause.l_side)),
    };
    let r_side = match &where_clause.r_side {
        Operand::Value(value) => value,
        _ => return Err(format!("Found invalid right side of condition: {:?}", where_clause.r_side)),
    };
    let column_value = table.get_column_from_row(row, &l_side);
    if column_value.get_type() == DataType::Null && r_side.get_type() == DataType::Null {
        return Ok(true);
    }
    else if column_value.get_type() == DataType::Null || r_side.get_type() == DataType::Null {
        return Ok(false);
    }
    if column_value.get_type() != r_side.get_type() {
        return Err(format!("Found different data types for column and value: {:?} and {:?}", column_value.get_type(), r_side.get_type()));
    }

    match where_clause.operator {
        Operator::Equals => {
            return Ok(*column_value == *r_side);
        },
        Operator::NotEquals => {
            return Ok(*column_value != *r_side);
        },
        _ => {
            match column_value.get_type() {
                DataType::Integer | DataType::Real | DataType::Text => {
                    match where_clause.operator {
                        Operator::LessThan => {
                            return Ok(*column_value < *r_side);
                        },
                        Operator::GreaterThan => {
                            return Ok(*column_value > *r_side);
                        },
                        Operator::LessEquals => {
                            return Ok(*column_value <= *r_side);
                        },
                        Operator::GreaterEquals => {
                            return Ok(*column_value >= *r_side);
                        },
                        _ => {
                            return Err(format!("Found invalid operator: {:?}", where_clause.operator));
                        },
                    }
                },
                _ => {
                    return Err(format!("Found invalid operator: {:?} for data type: {:?}", where_clause.operator, column_value.get_type()));
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition};

    #[test]
    fn matches_where_clause_returns_true_if_row_matches_where_clause() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {
                name:"id".to_string(),
                data_type:DataType::Integer, 
                constraints: vec![] 
            },
        ]);
        let row = vec![Value::Integer(1)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
    }

    #[test]
    fn matches_where_clause_returns_false_if_row_does_not_match_where_clause() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(2)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && !result.unwrap());
    }

    #[test]
    fn matches_where_clause_handles_different_data_types() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {
                name:"id".to_string(),
                data_type:DataType::Integer, 
                constraints: vec![] 
            },
        ]);
        let row = vec![Value::Integer(1)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Text("Fletcher".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_err());
        let expected_error = "Found different data types for column and value: Integer and Text";
        assert_eq!(expected_error, result.err().unwrap());
    }

    #[test]
    fn matches_where_clause_handles_different_operators() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(10)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterThan,r_side: Operand::Value(Value::Integer(0))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Integer(0))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::LessThan,r_side: Operand::Value(Value::Integer(20))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::LessEquals,r_side: Operand::Value(Value::Integer(20))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::NotEquals,r_side: Operand::Value(Value::Integer(10))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && !result.unwrap());
    }

    #[test]
    fn matches_where_clause_handles_string_comparison() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
        ]);
        let row = vec![Value::Text("lop".to_string())];
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Text("abc".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::LessEquals,r_side: Operand::Value(Value::Text("lop".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::GreaterThan,r_side: Operand::Value(Value::Text("xyz".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && !result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::LessThan,r_side: Operand::Value(Value::Text("abc".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && !result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::NotEquals,r_side: Operand::Value(Value::Text("abc".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Text("lop".to_string()))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && result.unwrap());
    }

    #[test]
    fn matches_where_clause_handles_null() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Null];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Integer(1))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_ok() && !result.unwrap());
    }

    #[test]
    fn matches_where_clause_handles_invalid_operator_for_data_type() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Blob, constraints: vec![] },
        ]);
        let row = vec![Value::Blob(vec![1, 2, 3])];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Blob(vec![1, 2, 3]))};
        let result = matches_where_clause(&table, &row, &where_clause);
        assert!(result.is_err());
        let expected_error = "Found invalid operator: GreaterEquals for data type: Blob";
        assert_eq!(expected_error, result.err().unwrap());
    }
}