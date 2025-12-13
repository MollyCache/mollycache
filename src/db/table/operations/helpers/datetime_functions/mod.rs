pub mod julian_day;
pub mod modifiers;
pub mod time_values;

use crate::db::table::core::value::Value;
use crate::db::table::operations::helpers::datetime_functions::julian_day::JulianDay;
use crate::db::table::operations::helpers::datetime_functions::modifiers::DateTimeModifier;
use crate::db::table::operations::helpers::datetime_functions::modifiers::parse_modifier;
use crate::db::table::operations::helpers::datetime_functions::time_values::parse_timevalue;
use crate::interpreter::ast::SelectableColumn;
use crate::interpreter::ast::SelectableStackElement;

pub fn build_julian_day(args: &Vec<SelectableColumn>) -> Result<JulianDay, String> {
    if args.is_empty() {
        return Err("Invalid DateTime function: no arguments".to_string());
    }
    let mut init_jdn = {
        let arg = &args[0];
        match &arg.selectables[0] {
            SelectableStackElement::Value(val) => parse_timevalue(val)?,
            _ => {
                return Err(format!(
                    "Invalid argument for datetime function: {:?}",
                    arg.selectables[0]
                ));
            }
        }
    };
    // ONLY SUPPORTS TIME MODIFIERS RN.
    for arg in args[1..].iter() {
        let arg = match &arg.selectables[0] {
            SelectableStackElement::Value(Value::Text(val)) => val.to_string(),
            _ => {
                return Err(format!(
                    "Invalid argument for datetime function: {:?}",
                    arg.selectables[0]
                ));
            }
        };
        let modifier = parse_modifier(&arg)?;
        match modifier {
            DateTimeModifier::JDNOffset(jd) => {
                *init_jdn.value_mut() += jd.value();
            }
            _ => {
                return Err(format!("NOT SUPPORTED YET: '{}'", arg));
            }
        }
    }
    Ok(init_jdn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::Value;

    #[test]
    fn test_build_julian_day_as_date_time() {
        let args = vec![SelectableColumn {
            selectables: vec![SelectableStackElement::Value(Value::Text(
                "2025-12-12".to_string(),
            ))],
            column_name: "text".to_string(),
        }];
        let result = build_julian_day(&args).unwrap().as_datetime();
        assert_eq!(result, "2025-12-12 00:00:00");
        let args = vec![SelectableColumn {
            selectables: vec![SelectableStackElement::Value(Value::Real(2461022.6789))],
            column_name: "real".to_string(),
        }];
        let result = build_julian_day(&args).unwrap().as_datetime();
        assert_eq!(result, "2025-12-13 04:17:36");
    }
}
