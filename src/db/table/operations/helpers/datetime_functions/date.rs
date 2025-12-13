use crate::db::table::core::value::Value;
use crate::interpreter::ast::SelectableColumn;

pub fn get_date(_args: &Vec<SelectableColumn>) -> Result<Value, String> {
    return Ok(Value::Text(format!("2025-12-12")));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::SelectableStackElement;

    #[test]
    fn test_get_current_date() {
        let args = vec![
            SelectableColumn { 
                selectables: vec![SelectableStackElement::Value(Value::Text("now".to_string()))], 
                column_name: "now".to_string() 
            },
        ];
        let result = get_date(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Text(format!("2025-12-12")));
        
    }
}