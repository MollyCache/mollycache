use crate::cli::ast::{Operator, WhereCondition, Operand};
use crate::db::table::{Table, Value, DataType};

// For now this function only supports one column = value where clause
pub fn matches_where_clause(table: &Table, row: &Vec<Value>, where_clause: &WhereCondition) -> bool {
    let l_side = match &where_clause.l_side {
        Operand::Identifier(column) => column,
        _ => return false,
    };
    let r_side = match &where_clause.r_side {
        Operand::Value(value) => value,
        _ => return false,
    };
    let column_value = table.get_column_from_row(row, &l_side);
    if column_value.get_type() != r_side.get_type() {
        return false;
    }

    match where_clause.operator {
        Operator::Equals => {
            return *column_value == *r_side;
        },
        Operator::NotEquals => {
            return *column_value != *r_side;
        },
        _ => {
            match column_value.get_type() {
                DataType::Integer | DataType::Real | DataType::Text => {
                    match where_clause.operator {
                        Operator::LessThan => {
                            return *column_value < *r_side;
                        },
                        Operator::GreaterThan => {
                            return *column_value > *r_side;
                        },
                        Operator::LessEquals => {
                            return *column_value <= *r_side;
                        },
                        Operator::GreaterEquals => {
                            return *column_value >= *r_side;
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
        assert!(matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_returns_false_if_row_does_not_match_where_clause() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(2)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Integer(1))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
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
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_different_operators() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(10)];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterThan,r_side: Operand::Value(Value::Integer(0))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Integer(0))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::LessThan,r_side: Operand::Value(Value::Integer(20))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::LessEquals,r_side: Operand::Value(Value::Integer(20))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::NotEquals,r_side: Operand::Value(Value::Integer(10))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_string_comparison() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
        ]);
        let row = vec![Value::Text("lop".to_string())];
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Text("abc".to_string()))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::LessEquals,r_side: Operand::Value(Value::Text("lop".to_string()))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::GreaterThan,r_side: Operand::Value(Value::Text("xyz".to_string()))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::LessThan,r_side: Operand::Value(Value::Text("abc".to_string()))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::NotEquals,r_side: Operand::Value(Value::Text("abc".to_string()))};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereCondition {l_side: Operand::Identifier("name".to_string()),operator:Operator::Equals,r_side: Operand::Value(Value::Text("lop".to_string()))};
        assert!(matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_null() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Null];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Integer(1))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_invalid_operator_for_data_type() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Blob, constraints: vec![] },
        ]);
        let row = vec![Value::Blob(vec![1, 2, 3])];
        let where_clause = WhereCondition {l_side: Operand::Identifier("id".to_string()),operator:Operator::GreaterEquals,r_side: Operand::Value(Value::Blob(vec![1, 2, 3]))};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }
}