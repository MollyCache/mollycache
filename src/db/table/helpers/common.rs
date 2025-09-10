use std::collections::HashMap;
use crate::db::table::{Table, Value, DataType};
use crate::interpreter::ast::{SelectableStack, SelectableStackElement, Operator, LogicalOperator, MathOperator, WhereStackElement, OrderByClause, LimitClause};
use crate::db::table::helpers::where_stack::matches_where_stack;
use crate::db::table::helpers::{order_by_clause::get_ordered_row_indicies, limit_clause::get_limited_rows};

pub fn validate_and_clone_row(table: &Table, row: &Vec<Value>) -> Result<Vec<Value>, String> {
    if row.len() != table.width() {
        return Err(format!("Rows have incorrect width"));
    }

    let mut row_values: Vec<Value> = vec![];
    for (i, value) in row.iter().enumerate() {
        if value.get_type() != table.columns[i].data_type && value.get_type() != DataType::Null {
            return Err(format!("Data type mismatch for column {}", table.columns[i].name));
        }
        row_values.push(row[i].clone());
    }
    return Ok(row_values);
}

pub fn get_row_columns_from_indicies(table: &Table, row_indicies: Vec<usize>, columns: Option<&SelectableStack>) -> Result<Vec<Vec<Value>>, String> {
    let mut rows: Vec<Vec<Value>> = vec![];
    for index in row_indicies {
        let row = table.rows[index].clone();
        if let Some(columns) = columns {
            rows.push(get_columns_from_row(table, &row, columns)?);
        }
        else {
            rows.push(validate_and_clone_row(table, &row)?);
        }
    }
    Ok(rows)
}

pub fn get_row_indicies_matching_where_clause(table: &Table, where_clause: &Option<Vec<WhereStackElement>>) -> Result<Vec<usize>, String> {
    if let Some(where_clause) = where_clause {
        let mut row_indicies: Vec<usize> = vec![];
        for (i, row) in table.rows.iter().enumerate() {
            if matches_where_stack(table, &row, &where_clause)? {
                row_indicies.push(i);
            }
        }
        return Ok(row_indicies);
    }
    else {
        return Ok((0..table.rows.len()).collect());
    }
}

pub fn get_columns_from_row(table: &Table, row: &Vec<Value>, selected_columns: &SelectableStack) -> Result<Vec<Value>, String> {
    let mut row_values: Vec<Value> = vec![];

    let mut column_values = HashMap::new();
    for (i, column) in table.columns.iter().enumerate() {
        if let Some(value) = row.get(i) {
            column_values.insert(column.name.to_string(), value);
        } else {
            return Err(format!("Row does not have the expected number of columns (expected: {}, got: {}", table.columns.len(), row.len()));
        }
    }

    let column_values = column_values;

    for selectable in &selected_columns.selectables {
        match selectable {
            SelectableStackElement::All => {
                for val in row {
                    row_values.push(val.clone());
                }
            },
            SelectableStackElement::Column(value) => {
                if let Some(value) = column_values.get(value) {
                    row_values.push((*value).clone());
                } else {
                    return Err(format!("Invalid column name: {}", value));
                }
            },
            SelectableStackElement::Value(value) => {
                row_values.push(value.clone());
            },
            SelectableStackElement::ValueList(list) => {
                // TODO: handle ValueList
            },
            SelectableStackElement::Function(func) => {
                // TODO: handle functions
            },
            SelectableStackElement::Operator(op) => {
                let res = match op {
                    Operator::Equals => pop_two_and_operate(|a, b| Ok(a == b), &mut row_values, None)?,
                    Operator::NotEquals => pop_two_and_operate(|a, b| Ok(a != b), &mut row_values, None)?,
                    Operator::LessThan => pop_two_and_operate(|a, b| Ok(a < b), &mut row_values, None)?,
                    Operator::GreaterThan => pop_two_and_operate(|a, b| Ok(a > b), &mut row_values, None)?,
                    Operator::LessEquals => pop_two_and_operate(|a, b| Ok(a <= b), &mut row_values, None)?,
                    Operator::GreaterEquals => pop_two_and_operate(|a, b| Ok(a >= b), &mut row_values, None)?,
                    // TODO: In, NotIn, Is, IsNot
                    _ => false,
                };
                // TODO: add Bool type
                row_values.push(Value::Integer(if res { 1 } else { 0 }));
            },
            SelectableStackElement::LogicalOperator(op) => {
                let res = match op {
                    // TODO: add Bool as Value
                    LogicalOperator::Not => pop_one_and_operate(|a| {
                        if let Value::Integer(val) = a {
                            Ok(val == 0)
                        } else {
                            Err("Unexpected type for NOT".to_string())
                        }
                    }, &mut row_values, None)?,
                    LogicalOperator::And => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(val1), Value::Integer(val2)) = (a, b) {
                            Ok(val1 != 0 && val2 != 1)
                        } else {
                            Err("Unexpected type(s) for AND".to_string())
                        }
                    }, &mut row_values, None)?,
                    LogicalOperator::Or => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(val1), Value::Integer(val2)) = (a, b) {
                            Ok(val1 == 1 || val2 == 1)
                        } else {
                            Err("Unexpected type(s) for OR".to_string())
                        }
                    }, &mut row_values, None)?,
                };
                row_values.push(Value::Integer(if res { 1 } else { 0 }));
            },
            SelectableStackElement::MathOperator(op) => {
                let res = match op {
                    MathOperator::Add => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                            Ok(Value::Integer(a_i + b_i))
                        } else if let (Some(a_f), Some(b_f)) = (a.as_f64(), b.as_f64()) {
                            Ok(Value::Real(a_f + b_f))
                        } else {
                            Err("Unexpected type(s) for ADD".to_string())
                        }
                    }, &mut row_values, None)?,
                    MathOperator::Subtract => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                            Ok(Value::Integer(a_i - b_i))
                        } else if let (Some(a_f), Some(b_f)) = (a.as_f64(), b.as_f64()) {
                            Ok(Value::Real(a_f - b_f))
                        } else {
                            Err("Unexpected type(s) for SUBTRACT".to_string())
                        }
                    }, &mut row_values, None)?,
                    MathOperator::Multiply => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                            Ok(Value::Integer(a_i * b_i))
                        } else if let (Some(a_f), Some(b_f)) = (a.as_f64(), b.as_f64()) {
                            Ok(Value::Real(a_f * b_f))
                        } else {
                            Err("Unexpected type(s) for MULTIPLY".to_string())
                        }
                    }, &mut row_values, None)?,
                    MathOperator::Divide => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                            Ok(Value::Integer(a_i / b_i))
                        } else if let (Some(a_f), Some(b_f)) = (a.as_f64(), b.as_f64()) {
                            Ok(Value::Real(a_f / b_f))
                        } else {
                            Err("Unexpected type(s) for DIVIDE".to_string())
                        }
                    }, &mut row_values, None)?,
                    MathOperator::Modulo => pop_two_and_operate(|a, b| {
                        if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                            Ok(Value::Integer(a_i % b_i))
                        } else {
                            Err("Unexpected type(s) for MODULO".to_string())
                        }
                    }, &mut row_values, None)?
                };
                row_values.push(res);
            }
        }
    }

    return Ok(row_values);
}

pub fn get_row_indicies_matching_clauses(table: &Table, where_clause: &Option<Vec<WhereStackElement>>, order_by_clause: &Option<Vec<OrderByClause>>, limit_clause: &Option<LimitClause>) -> Result<Vec<usize>, String> {
    // TODO: if LIMIT without ORDER BY, apply LIMIT at the beginning (if no WHERE) / after the WHERE
    let mut row_indicies = get_row_indicies_matching_where_clause(table, where_clause)?;

    if let Some(order_by_clause) = order_by_clause {
        row_indicies = get_ordered_row_indicies(table, row_indicies, &order_by_clause)?;
    }

    if let Some(limit_clause) = limit_clause {
        let result = get_limited_rows(row_indicies, &limit_clause)?;
        return Ok(result.to_vec());
    }

    return Ok(row_indicies);
}

fn pop_one_and_operate<F, R>(f: F, values: &mut Vec<Value>, err: Option<String>) -> Result<R, String> where F: Fn(Value) -> Result<R, String> {
    if let Some(val) = values.pop() {
        return f(val);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}

fn pop_two_and_operate<F, R>(f: F, values: &mut Vec<Value>, err: Option<String>) -> Result<R, String> where F: Fn(Value, Value) -> Result<R, String> {
    if let Some(second) = values.pop() && let Some(first) = values.pop() {
        return f(first, second);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}
