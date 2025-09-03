use crate::cli::ast::{Operator, WhereTreeEdge};
use crate::db::table::{Table, Value, DataType};

pub fn matches_where_clause(table: &Table, row: &Vec<Value>, where_clause: &WhereTreeEdge) -> bool {
    let column_value = table.get_column_from_row(row, &where_clause.column);
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
                DataType::Integer | DataType::Real | DataType::Text => {
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
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::Equals,value:Value::Integer(1)};
        assert!(matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_returns_false_if_row_does_not_match_where_clause() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(2)];
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::Equals,value:Value::Integer(1)};
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
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::Equals,value:Value::Text("Fletcher".to_string())};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_different_operators() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Integer(10)];
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::GreaterThan,value:Value::Integer(0)};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::GreaterEquals,value:Value::Integer(0)};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::LessThan,value:Value::Integer(20)};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::LessEquals,value:Value::Integer(20)};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::NotEquals,value:Value::Integer(10)};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_string_comparison() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"name".to_string(),data_type:DataType::Text, constraints: vec![] },
        ]);
        let row = vec![Value::Text("lop".to_string())];
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::GreaterEquals,value:Value::Text("abc".to_string())};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::LessEquals,value:Value::Text("lop".to_string())};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::GreaterThan,value:Value::Text("xyz".to_string())};
        assert!(!matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::LessThan,value:Value::Text("abc".to_string())};
        assert!(!matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::NotEquals,value:Value::Text("abc".to_string())};
        assert!(matches_where_clause(&table, &row, &where_clause));
        let where_clause = WhereTreeEdge {column:"name".to_string(),operator:Operator::Equals,value:Value::Text("lop".to_string())};
        assert!(matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_null() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Integer, constraints: vec![] },
        ]);
        let row = vec![Value::Null];
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::GreaterEquals,value:Value::Integer(1)};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }

    #[test]
    fn matches_where_clause_handles_invalid_operator_for_data_type() {
        let table = Table::new("users".to_string(), vec![
            ColumnDefinition {name:"id".to_string(),data_type:DataType::Blob, constraints: vec![] },
        ]);
        let row = vec![Value::Blob(vec![1, 2, 3])];
        let where_clause = WhereTreeEdge {column:"id".to_string(),operator:Operator::GreaterEquals,value:Value::Blob(vec![1, 2, 3])};
        assert!(!matches_where_clause(&table, &row, &where_clause));
    }
}