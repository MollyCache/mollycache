use std::collections::HashMap;
use std::collections::HashSet;

use crate::db::table::core::{row::Row, table::Table, value::DataType, value::Value};
use crate::db::table::operations::helpers::order_by_clause::apply_order_by_from_precomputed;
use crate::interpreter::ast::{
    LimitClause, LogicalOperator, MathOperator, Operator, OrderByClause, SelectableColumn,
    SelectableStackElement, FunctionName,
};
use crate::db::table::operations::helpers::datetime_functions::date::get_date;

pub fn validate_and_clone_row(table: &Table, row: &Row) -> Result<Row, String> {
    if row.len() != table.width()? {
        return Err(format!("Rows have incorrect width"));
    }

    let mut row_values: Row = Row(vec![]);
    for (i, value) in row.iter().enumerate() {
        if value.get_type() != table.get_columns()?[i].data_type
            && value.get_type() != DataType::Null
        {
            return Err(format!(
                "Data type mismatch for column {}",
                table.get_columns()?[i].name
            ));
        }
        row_values.push(row[i].clone());
    }
    return Ok(row_values);
}

pub fn get_columns(
    table: &Table,
    row: &Row,
    selected_columns: &Vec<SelectableColumn>,
    computed_columns: Option<&Row>,
    aliases_to_indexes: Option<&HashMap<String, usize>>,
) -> Result<Row, String> {
    let mut columns = vec![];
    for col in selected_columns {
        if col
            .selectables
            .first()
            .is_some_and(|elem| elem == &SelectableStackElement::All)
        {
            for val in row.iter() {
                // TODO: can we do this?
                columns.push(val.clone());
            }
        } else {
            columns.push(get_column(
                table,
                row,
                col,
                computed_columns,
                aliases_to_indexes,
            )?);
        }
    }
    return Ok(Row(columns));
}

pub fn get_column(
    table: &Table,
    row: &Row,
    selected_column: &SelectableColumn,
    computed_columns: Option<&Row>,
    aliases_to_indexes: Option<&HashMap<String, usize>>,
) -> Result<Value, String> {
    // Does NOT handle SelectableStackElement::All, since only returns one Value
    let mut row_values: Row = Row(vec![]);

    let mut column_values = HashMap::new();
    for (i, column) in table.get_column_names()?.into_iter().enumerate() {
        if let Some(value) = row.get(i) {
            column_values.insert(column, value);
        } else {
            return Err(format!(
                "Row does not have the expected number of columns (expected: {}, got: {}",
                table.get_columns()?.len(),
                row.len()
            ));
        }
    }

    let column_values = column_values;

    for selectable in &selected_column.selectables {
        match selectable {
            SelectableStackElement::All => {
                for val in row.iter() {
                    // TODO: can we do this?
                    row_values.push(val.clone());
                }
            }
            SelectableStackElement::Column(value) => {
                if let Some(computed) = computed_columns
                    && let Some(map) = aliases_to_indexes
                    && let Some(index) = map.get(value)
                {
                    if let Some(val) = computed.get(*index) {
                        row_values.push((*val).clone());
                    } else {
                        return Err(format!("Couldn't resolve alias: {}", value));
                    }
                } else if let Some(val) = column_values.get(value) {
                    row_values.push((*val).clone());
                } else {
                    return Err(format!("Invalid column name: {}", value));
                }
            }
            SelectableStackElement::Value(value) => {
                row_values.push(value.clone());
            }
            SelectableStackElement::ValueList(_) => {
                // TODO: handle ValueList
            }

            // THIS IS SPECIFIC TO SCALAR FUNCTIONS i.e. (date, time)
            SelectableStackElement::Function(func) => {
                let args = &func.arguments;
                let res = match func.name {
                    FunctionName::Date => get_date(args)?,
                    _ => return Err(format!("Unsupported function: {:?}", func.name)),
                };
                row_values.push(res);
            }
            SelectableStackElement::Operator(op) => {
                let res = match op {
                    Operator::Equals => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 == val2),
                            _ => Ok(a == b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::NotEquals => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 != val2),
                            _ => Ok(a != b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::LessThan => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 < val2),
                            _ => Ok(a < b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::GreaterThan => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 > val2),
                            _ => Ok(a > b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::LessEquals => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 <= val2),
                            _ => Ok(a <= b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::GreaterEquals => pop_two_and_operate(
                        |a, b| match (a.numeric_to_f64(), b.numeric_to_f64()) {
                            (Some(val1), Some(val2)) => Ok(val1 >= val2),
                            _ => Ok(a >= b),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::Is => pop_two_and_operate(
                        |a, b| match (a, b) {
                            (Value::Null, Value::Null) => Ok(true),
                            (Value::Null, _) | (_, Value::Null) => Ok(false),
                            (first, second) => Ok(first == second),
                        },
                        &mut row_values,
                        None,
                    )?,
                    Operator::IsNot => pop_two_and_operate(
                        |a, b| match (a, b) {
                            (Value::Null, Value::Null) => Ok(false),
                            (Value::Null, _) | (_, Value::Null) => Ok(true),
                            (first, second) => Ok(first != second),
                        },
                        &mut row_values,
                        None,
                    )?,
                    // TODO: In, NotIn
                    _ => false,
                };
                // TODO: add Bool type
                row_values.push(Value::Integer(if res { 1 } else { 0 }));
            }
            SelectableStackElement::LogicalOperator(op) => {
                let res = match op {
                    // TODO: add Bool as Value
                    LogicalOperator::Not => pop_one_and_operate(
                        |a| {
                            if let Value::Integer(val) = a {
                                Ok(val == 0)
                            } else {
                                Err("Unexpected type for NOT".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    LogicalOperator::And => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(val1), Value::Integer(val2)) = (a, b) {
                                Ok(val1 != 0 && val2 != 0)
                            } else {
                                Err("Unexpected type(s) for AND".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    LogicalOperator::Or => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(val1), Value::Integer(val2)) = (a, b) {
                                Ok(val1 != 0 || val2 != 0)
                            } else {
                                Err("Unexpected type(s) for OR".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                };
                row_values.push(Value::Integer(if res { 1 } else { 0 }));
            }
            SelectableStackElement::MathOperator(op) => {
                let res = match op {
                    MathOperator::Add => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                                Ok(Value::Integer(a_i + b_i))
                            } else if let (Some(a_f), Some(b_f)) =
                                (a.numeric_to_f64(), b.numeric_to_f64())
                            {
                                Ok(Value::Real(a_f + b_f))
                            } else {
                                Err("Unexpected type(s) for ADD".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    MathOperator::Subtract => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                                Ok(Value::Integer(a_i - b_i))
                            } else if let (Some(a_f), Some(b_f)) =
                                (a.numeric_to_f64(), b.numeric_to_f64())
                            {
                                Ok(Value::Real(a_f - b_f))
                            } else {
                                Err("Unexpected type(s) for SUBTRACT".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    MathOperator::Multiply => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                                Ok(Value::Integer(a_i * b_i))
                            } else if let (Some(a_f), Some(b_f)) =
                                (a.numeric_to_f64(), b.numeric_to_f64())
                            {
                                Ok(Value::Real(a_f * b_f))
                            } else {
                                Err("Unexpected type(s) for MULTIPLY".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    MathOperator::Divide => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                                if *b_i == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                Ok(Value::Integer(a_i / b_i))
                            } else if let (Some(a_f), Some(b_f)) =
                                (a.numeric_to_f64(), b.numeric_to_f64())
                            {
                                if b_f == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                Ok(Value::Real(a_f / b_f))
                            } else {
                                Err("Unexpected type(s) for DIVIDE".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                    MathOperator::Modulo => pop_two_and_operate(
                        |a, b| {
                            if let (Value::Integer(a_i), Value::Integer(b_i)) = (&a, &b) {
                                if *b_i == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                Ok(Value::Integer(a_i % b_i))
                            } else {
                                Err("Unexpected type(s) for MODULO".to_string())
                            }
                        },
                        &mut row_values,
                        None,
                    )?,
                };
                row_values.push(res);
            }
        }
    }

    if row_values.len() != 1 {
        return Err("Selected column does not result in exactly one value".to_string());
    }

    // TODO: pretty inefficient cloning. This function is called a LOT. Maybe, since now we know its length is 1, we can pop to take ownership?
    return Ok(row_values[0].clone());
}

// Used for UPDATE and DELETE. Not get_row_indicies_matching_clauses used for INSERT, since it possibly contains DISTINCT, in which case we need the actual evaluated SELECT values, not just the indices
pub fn get_row_indicies_matching_clauses(
    table: &Table,
    where_clause: &Option<SelectableColumn>,
    order_by_clause: &Option<OrderByClause>,
    limit_clause: &Option<LimitClause>,
) -> Result<Vec<usize>, String> {
    let mut indices = vec![];
    let mut order_by_columns_precomputed = vec![];
    let (limit, offset) = limit_clause.as_ref().map_or((-1, 0), |stmt| {
        (stmt.limit as i64, stmt.offset.map_or(0, |val| val))
    });

    for (i, row) in table
        .iter()
        .skip(if order_by_clause.is_none() { offset } else { 0 })
        .enumerate()
    {
        if limit != -1 && indices.len() as i64 >= limit && order_by_clause.is_none() {
            break;
        } else if let Some(stmt) = where_clause {
            if let Value::Integer(val) = get_column(table, row, stmt, None, None)? {
                if val == 0 {
                    continue;
                }
            } else {
                return Err("WHERE condition did not return a boolean".to_string());
            }
        }

        // Fix: When skipping rows without ORDER BY, enumerate() resets index to 0
        // We need to add the offset back to get the actual table index
        let actual_index = i + if order_by_clause.is_none() { offset } else { 0 };
        indices.push(actual_index);
        if let Some(stmt) = order_by_clause {
            // UPDATE and DELETE only, so not reading from any alias table
            order_by_columns_precomputed.push(get_columns(table, row, &stmt.columns, None, None)?);
        }
    }

    if let Some(stmt) = order_by_clause {
        apply_order_by_from_precomputed(&mut indices, order_by_columns_precomputed, 0, stmt);
        if limit != -1 || offset != 0 {
            // If offset exceeds the result size, return empty set (SQLite-compatible behavior)
            if offset >= indices.len() {
                indices = vec![];
            } else {
                let end = if (limit == -1) || (offset + limit as usize > indices.len()) {
                    indices.len()
                } else {
                    offset + limit as usize
                };
                indices = indices[offset..end].to_vec();
            }
        }
    }

    Ok(indices)
}

pub fn remove_duplicate_rows(rows: Vec<Row>) -> Vec<Row> {
    let set = rows.into_iter().collect::<HashSet<Row>>();
    let result = set.into_iter().collect::<Vec<Row>>();
    return result;
}

fn pop_one_and_operate<F, R>(f: F, values: &mut Row, err: Option<String>) -> Result<R, String>
where
    F: Fn(Value) -> Result<R, String>,
{
    if let Some(val) = values.pop() {
        return f(val);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}

fn pop_two_and_operate<F, R>(f: F, values: &mut Row, err: Option<String>) -> Result<R, String>
where
    F: Fn(Value, Value) -> Result<R, String>,
{
    if let Some(second) = values.pop()
        && let Some(first) = values.pop()
    {
        return f(first, second);
    } else {
        return Err(err.unwrap_or(format!("Not enough values to compare with operator")));
    }
}
