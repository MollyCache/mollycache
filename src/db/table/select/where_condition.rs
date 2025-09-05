use crate::db::table::{Table, Value};
use crate::cli::ast::{Operator, Operand, WhereCondition};
use crate::db::table::DataType;


// This file holds the logic for whether a row matches a where condition.
pub fn matches_where_clause(table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> Result<bool, String> {
    match where_clause.operator {
        Operator::In | Operator::NotIn => {
            todo!(); // Todo handle IN and NOT IN.
        },
        _ => {},
    }
    let l_side = operand_to_value(table, row, &where_clause.l_side)?;
    let r_side = operand_to_value(table, row, &where_clause.r_side)?;
    if l_side.get_type() == DataType::Null && r_side.get_type() == DataType::Null {
        return Ok(true);
    }
    else if l_side.get_type() == DataType::Null || r_side.get_type() == DataType::Null {
        return Ok(false);
    }
    if l_side.get_type() != r_side.get_type() {
        return Err(format!("Found different data types for column and value: {:?} and {:?}", l_side.get_type(), r_side.get_type()));
    }

    match where_clause.operator {
        Operator::Equals => {
            return Ok(*l_side == *r_side);
        },
        Operator::NotEquals => {
            return Ok(*l_side != *r_side);
        },
        _ => {
            match l_side.get_type() {
                DataType::Integer | DataType::Real | DataType::Text => {
                    match where_clause.operator {
                        Operator::LessThan => {
                            return Ok(*l_side < *r_side);
                        },
                        Operator::GreaterThan => {
                            return Ok(*l_side > *r_side);
                        },
                        Operator::LessEquals => {
                            return Ok(*l_side <= *r_side);
                        },
                        Operator::GreaterEquals => {
                            return Ok(*l_side >= *r_side);
                        },
                        _ => {
                            return Err(format!("Found invalid operator: {:?}", where_clause.operator));
                        },
                    }
                },
                _ => {
                    return Err(format!("Found invalid operator: {:?} for data type: {:?}", where_clause.operator, l_side.get_type()));
                },
            }
        }
    }
}

fn operand_to_value<'a>(table: &'a Table, row: &'a Vec<Value>, operand: &'a Operand) -> Result<&'a Value, String> {
    match operand {
        Operand::Value(value) => Ok(value),
        Operand::Identifier(column) => {
            if !table.has_column(column) {
                return Err(format!("Column {} does not exist in table {}", column, table.name));
            }
            Ok(table.get_column_from_row(row, column))
        },
        _ => Err(format!("Found invalid operand: {:?}", operand)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::{Table, Value, DataType, ColumnDefinition};
    use crate::cli::ast::{Operator, Operand, WhereCondition};

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