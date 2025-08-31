use std::cmp::min;

use crate::cli::ast::LimitClause;
use crate::db::table::Value;


pub fn get_limited_rows<'a>(rows: Vec<Vec<Value>>, limit_clause: &LimitClause) -> Result<Vec<Vec<Value>>, String> {
    let mut index = 0;
    if let Some(offset) = &limit_clause.offset && let Value::Integer(offset) = offset {
        index = *offset as usize;
    }
    if index >= rows.len()  {
        return Ok(vec![]);
    }
    
    let limit = match limit_clause.limit {
        Value::Integer(limit) => {
            if limit < 0 {
                rows.len()
            } else {
                min((limit as usize)+index, rows.len())
            }
        },
        _ => return Err("Limit must be an integer".to_string()), // The parser should have already validated this
    };

    let mut limited_rows: Vec<Vec<Value>> = vec![];
    for i in index..limit {
        limited_rows.push(rows[i].clone());
    }
    return Ok(limited_rows);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn default_rows() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
            vec![Value::Integer(5)],
            vec![Value::Integer(6)],
            vec![Value::Integer(7)],
            vec![Value::Integer(8)],
            vec![Value::Integer(9)],
            vec![Value::Integer(10)],
        ]
    }

    fn generate_limit_clause(limit: i64, offset: Option<i64>) -> LimitClause {
        LimitClause {
            limit: Value::Integer(limit as i64),
            offset: offset.map(|offset| Value::Integer(offset as i64)),
        }
    }

    #[test]
    fn no_offset_and_limit_is_equal_to_rows_length() {
        let limit_clause = generate_limit_clause(10, None);
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }

    #[test]
    fn no_offset_and_limit_is_greater_than_rows_length() {
        let limit_clause = generate_limit_clause(15, None);
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }

    #[test]
    fn no_offset_and_limit_is_less_than_rows_length() {
        let limit_clause = generate_limit_clause(5, None);
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
            vec![Value::Integer(5)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn no_offset_and_negative_limit_returns_all_rows() {
        let limit_clause = generate_limit_clause(-1, None);
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }
    
    #[test]
    fn offset_and_limit_is_generated_correctly() {
        let limit_clause = generate_limit_clause(5, Some(1));
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected = vec![
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
            vec![Value::Integer(5)],
            vec![Value::Integer(6)],
        ];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn offset_is_greater_than_rows_length_returns_empty_rows() {
        let limit_clause = generate_limit_clause(5, Some(10));
        let result = get_limited_rows(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected: Vec<Vec<Value>> = vec![];
        assert_eq!(expected, result.unwrap());
    }
}