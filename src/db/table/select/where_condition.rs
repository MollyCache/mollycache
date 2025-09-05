use crate::db::table::{Table, Value};
use crate::cli::ast::{Operator, Operand, WhereCondition};
use crate::db::table::DataType;


// This file holds the logic for whether a row matches a where condition

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