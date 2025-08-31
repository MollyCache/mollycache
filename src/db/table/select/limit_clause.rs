use crate::cli::ast::LimitClause;
use crate::db::table::Value;


pub fn get_limited_rows(rows: Vec<Vec<Value>>, limit_clause: &LimitClause) -> Result<Vec<Vec<Value>>, String> {
    let mut index = 0;
    if let Some(offset) = &limit_clause.offset && let Value::Integer(offset) = offset {
        index = *offset as usize;
    }
    if index > rows.len()  {
        return Ok(rows);
    }
    
    let limit = match limit_clause.limit {
        Value::Integer(limit) => {
            if limit < 0 {
                rows.len()
            } else {
                limit as usize
            }
        },
        _ => {
            return Err("Limit must be an integer".to_string());
        }
    };

    // let limited_rows: Vec<Vec<Value>> = rows.into_iter().skip(index as usize).take(limit as usize).collect();
    return Ok(limited_rows);
}
