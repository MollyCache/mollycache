use std::collections::HashMap;
use std::collections::HashSet;

use crate::db::table::helpers::order_by_clause::apply_order_by_from_precomputed;
use crate::db::table::{Table, DataType, Row, Value};
use crate::interpreter::ast::{SelectableStack, SelectableStackElement, Operator, LogicalOperator, MathOperator, WhereStackElement, OrderByClause, LimitClause};
use crate::db::table::helpers::where_clause::row_matches_where_stack;

pub fn validate_and_clone_row(table: &Table, row: &Row) -> Result<Row, String> {
    if row.len() != table.width() {
        return Err(format!("Rows have incorrect width"));
    }

    let mut row_values: Row = Row(vec![]);
    for (i, value) in row.iter().enumerate() {
        if value.get_type() != table.columns[i].data_type && value.get_type() != DataType::Null {
            return Err(format!("Data type mismatch for column {}", table.columns[i].name));
        }
        row_values.push(row[i].clone());
    }
    return Ok(row_values);
}

pub fn get_columns_from_row(table: &Table, row: &Row, selected_columns: &SelectableStack) -> Result<Row, String> {
    let mut row_values: Row = Row(vec![]);

    let mut column_values = HashMap::new();
    for (i, column) in table.get_columns().into_iter().enumerate() {
        if let Some(value) = row.get(i) {
            column_values.insert(column, value);
        } else {
            return Err(format!("Row does not have the expected number of columns (expected: {}, got: {}", table.columns.len(), row.len()));
        }
    }

    let column_values = column_values;

    for selectable in &selected_columns.selectables {
        match selectable {
            SelectableStackElement::All => {
                for val in row.iter() {
                    // TODO: can we do this?
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

// Used for UPDATE and DELETE. Notget_row_indicies_matching_clauses used for INSERT, since it possibly contains DISTINCT, in which case we need the actual evaluated SELECT values, not just the indices
pub fn get_row_indicies_matching_clauses(table: &Table, where_clause: &Option<Vec<WhereStackElement>>, order_by_clause: &Option<OrderByClause>, limit_clause: &Option<LimitClause>) -> Result<Vec<usize>, String> {
    let mut indices = vec![];
    let mut order_by_columns_precomputed = vec![];
    let (limit, offset) = limit_clause.as_ref().map_or(
        (-1, 0),
        |stmt| (
            stmt.limit as i64,
            stmt.offset.map_or(0, |val| val)
        )
    );


    for (i, row) in table.iter().skip(
        if order_by_clause.is_none() {offset} else {0}
    ).enumerate() {
        if limit != -1 && indices.len() as i64 >= limit && order_by_clause.is_none() {
            break;
        } else if where_clause.as_ref().map_or_else(|| Ok(true), |stmt| row_matches_where_stack(table, row, &stmt))? {
            indices.push(i);
            if let Some(stmt) = order_by_clause {
                order_by_columns_precomputed.push(get_columns_from_row(table, row, &stmt.columns)?);
            }
        }
    }

    if let Some(stmt) = order_by_clause {
        apply_order_by_from_precomputed(&mut indices, order_by_columns_precomputed, 0, stmt);
        if limit != -1 || offset != 0 {
            let end = if limit == -1 {indices.len()} else {offset + limit as usize};
            indices = indices[offset..end].to_vec();
        }
    }

    Ok(indices)
}

pub fn remove_duplicate_rows(rows: Vec<Row>) -> Vec<Row> {
    let set = rows.into_iter().collect::<HashSet<Row>>();
    let result = set.into_iter().collect::<Vec<Row>>();
    return result;
}

fn pop_one_and_operate<F, R>(f: F, values: &mut Row, err: Option<String>) -> Result<R, String> where F: Fn(Value) -> Result<R, String> {
    if let Some(val) = values.pop() {
        return f(val);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}

fn pop_two_and_operate<F, R>(f: F, values: &mut Row, err: Option<String>) -> Result<R, String> where F: Fn(Value, Value) -> Result<R, String> {
    if let Some(second) = values.pop() && let Some(first) = values.pop() {
        return f(first, second);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}
