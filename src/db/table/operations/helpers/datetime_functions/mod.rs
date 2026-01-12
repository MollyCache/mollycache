pub mod julian_day;
pub mod modifiers;
pub mod time_values;

use crate::db::table::core::value::Value;
use crate::db::table::operations::helpers::datetime_functions::julian_day::{
    days_in_month, JulianDay,
};
use crate::db::table::operations::helpers::datetime_functions::modifiers::{
    parse_modifier, DateTimeModifier,
};
use crate::db::table::operations::helpers::datetime_functions::time_values::parse_timevalue;
use crate::interpreter::ast::SelectableColumn;
use crate::interpreter::ast::SelectableStackElement;

pub fn build_julian_day(args: &Vec<SelectableColumn>) -> Result<JulianDay, String> {
    if args.is_empty() {
        return Err("Invalid DateTime function: no arguments".to_string());
    }
    let mut current_jdn = {
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

    for arg in args[1..].iter() {
        let arg_str = match &arg.selectables[0] {
            SelectableStackElement::Value(Value::Text(val)) => val.to_string(),
            _ => {
                // Modifiers must be text strings
                return Err(format!(
                    "Invalid modifier for datetime function: {:?}",
                    arg.selectables[0]
                ));
            }
        };
        let modifier = parse_modifier(&arg_str)?;
        current_jdn = apply_modifier(current_jdn, modifier)?;
    }
    Ok(current_jdn)
}

fn apply_modifier(jd: JulianDay, modifier: DateTimeModifier) -> Result<JulianDay, String> {
    match modifier {
        DateTimeModifier::AddDays(days) => Ok(JulianDay::new(jd.value() + days)),
        DateTimeModifier::AddHours(hours) => Ok(JulianDay::new(jd.value() + hours / 24.0)),
        DateTimeModifier::AddMinutes(minutes) => Ok(JulianDay::new(jd.value() + minutes / 1440.0)),
        DateTimeModifier::AddSeconds(seconds) => Ok(JulianDay::new(jd.value() + seconds / 86400.0)),
        DateTimeModifier::AddMonths(months) => add_months(jd, months as i64),
        DateTimeModifier::AddYears(years) => add_years(jd, years as i64),
        DateTimeModifier::ShiftDate {
            years,
            months,
            days,
        } => {
            let jd = add_years(jd, years as i64)?;
            let jd = add_months(jd, months as i64)?;
            Ok(JulianDay::new(jd.value() + days))
        }
        DateTimeModifier::ShiftTime {
            hours,
            minutes,
            seconds,
        } => {
            let offset_days = hours / 24.0 + minutes / 1440.0 + seconds / 86400.0;
            Ok(JulianDay::new(jd.value() + offset_days))
        }
        DateTimeModifier::ShiftDateTime {
            years,
            months,
            days,
            hours,
            minutes,
            seconds,
        } => {
            let jd = add_years(jd, years as i64)?;
            let jd = add_months(jd, months as i64)?;
            let jd = JulianDay::new(jd.value() + days);
            let offset_days = hours / 24.0 + minutes / 1440.0 + seconds / 86400.0;
            Ok(JulianDay::new(jd.value() + offset_days))
        }
        DateTimeModifier::StartOfMonth => {
            let (y, m, _, _, _, _, _) = jd.to_calendar_components();
            Ok(JulianDay::new_from_datetime_vals(
                y as f64, m as f64, 1.0, 0.0, 0.0, 0.0, 0.0,
            ))
        }
        DateTimeModifier::StartOfYear => {
            let (y, _, _, _, _, _, _) = jd.to_calendar_components();
            Ok(JulianDay::new_from_datetime_vals(
                y as f64, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0,
            ))
        }
        DateTimeModifier::StartOfDay => {
            let (y, m, d, _, _, _, _) = jd.to_calendar_components();
            Ok(JulianDay::new_from_datetime_vals(
                y as f64, m as f64, d as f64, 0.0, 0.0, 0.0, 0.0,
            ))
        }
        DateTimeModifier::Weekday(target_weekday) => {
            let current_weekday = ((jd.value() + 1.5).floor() as i64) % 7;
            let days_to_add = (target_weekday - current_weekday + 7) % 7;
            let days_to_add = if days_to_add == 0 { 7 } else { days_to_add };
            Ok(JulianDay::new(jd.value() + days_to_add as f64))
        }
        DateTimeModifier::UnixEpoch => {
            // Treat the CURRENT value as a unix timestamp (seconds since 1970)
            let unix_seconds = jd.value();
            let jdn = (unix_seconds / 86400.0) + 2440587.5;
            Ok(JulianDay::new(jdn))
        }
        DateTimeModifier::JulianDay => {
            // No-op? Or assume current is JDN?
            // "The julianday modifier interprets the ... argument as a Julian day number."
            // Since we already parse as JDN, this is usually a no-op unless we parsed it as something else?
            // "julianday" usually forces the input to be treated as JDN.
            // But parse_timevalue parses numbers as JDN by default if they are numbers.
            // If the user did `datetime('2461022.5', 'julianday')` -> it's redundant but safe.
            Ok(jd)
        }
        DateTimeModifier::Auto => Ok(jd), // Default behavior
        DateTimeModifier::Localtime => {
            // Not supported in pure std without crates, doing no-op for now.
            // Could implement a rudimentary offset if we knew env TZ but we don't.
            Ok(jd)
        }
        DateTimeModifier::Utc => {
            // Assuming we are already UTC or no-op.
            Ok(jd)
        }
        _ => Err("Modifier not implemented".to_string()),
    }
}

fn add_months(jd: JulianDay, months: i64) -> Result<JulianDay, String> {
    let (mut year, mut month, day, hour, minute, second, subsecond) = jd.to_calendar_components();
    
    // Normalize months
    month += months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }

    let max_days = days_in_month(year, month);
    let day = if day > max_days { max_days } else { day };

    Ok(JulianDay::new_from_datetime_vals(
        year as f64,
        month as f64,
        day as f64,
        hour as f64,
        minute as f64,
        second as f64,
        subsecond,
    ))
}

fn add_years(jd: JulianDay, years: i64) -> Result<JulianDay, String> {
    let (year, month, day, hour, minute, second, subsecond) = jd.to_calendar_components();
    let new_year = year + years;
    let max_days = days_in_month(new_year, month);
    let day = if day > max_days { max_days } else { day };

    Ok(JulianDay::new_from_datetime_vals(
        new_year as f64,
        month as f64,
        day as f64,
        hour as f64,
        minute as f64,
        second as f64,
        subsecond,
    ))
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
    }

    #[test]
    fn test_modifiers_add_months() {
        // Jan 31 + 1 month -> Feb 28 (non-leap)
        let args = vec![
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "2025-01-31".to_string(),
                ))],
                column_name: "date".to_string(),
            },
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "+1 month".to_string(),
                ))],
                column_name: "mod".to_string(),
            },
        ];
        let result = build_julian_day(&args).unwrap().as_date();
        assert_eq!(result, "2025-02-28");
    }

    #[test]
    fn test_modifiers_start_of() {
         let args = vec![
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "2025-12-12 15:30:45".to_string(),
                ))],
                column_name: "date".to_string(),
            },
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "start of month".to_string(),
                ))],
                column_name: "mod".to_string(),
            },
        ];
        let result = build_julian_day(&args).unwrap().as_datetime();
        assert_eq!(result, "2025-12-01 00:00:00");

        let args = vec![
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "2025-12-12 15:30:45".to_string(),
                ))],
                column_name: "date".to_string(),
            },
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "start of year".to_string(),
                ))],
                column_name: "mod".to_string(),
            },
        ];
        let result = build_julian_day(&args).unwrap().as_datetime();
        assert_eq!(result, "2025-01-01 00:00:00");
    }

    #[test]
    fn test_weekday_modifier() {
         // 2025-01-12 is Sunday
         let args = vec![
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "2025-01-12".to_string(),
                ))],
                column_name: "date".to_string(),
            },
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "weekday 1".to_string(),
                ))], // Next Monday
                column_name: "mod".to_string(),
            },
        ];
        let result = build_julian_day(&args).unwrap().as_date();
        assert_eq!(result, "2025-01-13");

        // Sunday to Sunday
        let args = vec![
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "2025-01-12".to_string(),
                ))],
                column_name: "date".to_string(),
            },
            SelectableColumn {
                selectables: vec![SelectableStackElement::Value(Value::Text(
                    "weekday 0".to_string(),
                ))], // Next Sunday
                column_name: "mod".to_string(),
            },
        ];
        let result = build_julian_day(&args).unwrap().as_date();
        assert_eq!(result, "2025-01-19");
    }
}