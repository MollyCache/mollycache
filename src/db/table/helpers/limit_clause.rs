use std::cmp::min;

use crate::cli::ast::LimitClause;
use crate::db::table::Value;


pub fn get_limited_row_indicies(rows: Vec<usize>, limit_clause: &LimitClause) -> Result<Vec<usize>, String> {
    let mut index: usize = 0;
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

    let mut limited_rows: Vec<usize> = vec![];
    for i in index..limit {
        limited_rows.push(rows[i]);
    }
    return Ok(limited_rows);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn default_rows() -> Vec<usize> {
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
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
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }

    #[test]
    fn no_offset_and_limit_is_greater_than_rows_length() {
        let limit_clause = generate_limit_clause(15, None);
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }

    #[test]
    fn no_offset_and_limit_is_less_than_rows_length() {
        let limit_clause = generate_limit_clause(5, None);
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected = vec![0, 1, 2, 3, 4];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn no_offset_and_negative_limit_returns_all_rows() {
        let limit_clause = generate_limit_clause(-1, None);
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        assert_eq!(default_rows(), result.unwrap());
    }
    
    #[test]
    fn offset_and_limit_is_generated_correctly() {
        let limit_clause = generate_limit_clause(5, Some(1));
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected = vec![1, 2, 3, 4, 5];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn offset_is_greater_than_rows_length_returns_empty_rows() {
        let limit_clause = generate_limit_clause(5, Some(10));
        let result = get_limited_row_indicies(default_rows(), &limit_clause);
        assert!(result.is_ok());
        let expected: Vec<usize> = vec![];
        assert_eq!(expected, result.unwrap());
    }
}