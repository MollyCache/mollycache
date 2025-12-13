use crate::db::table::operations::helpers::datetime_functions::julian_day::JulianDay;

#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeModifier {
    JDNOffset(JulianDay),
    Ceiling,
    Floor,
    StartOfMonth,
    StartOfYear,
    StartOfDay,
    Weekday(i64),
    UnixEpoch,
    JulianDay,
    Auto,
    Localtime,
    Utc,
    Subsecond,
}

// Parsing here is done according to the SQLite documentation for date and time function modifiers.
// https://sqlite.org/lang_datefunc.html see section 3
pub fn parse_modifier(modifier: &str) -> Result<DateTimeModifier, String> {
    // Parse 'weekday N' format.
    if let Some(value) = modifier.strip_prefix("weekday ") {
        let value = value.trim();
        if value.is_empty() {
            return Err("Weekday modifier requires a numeric argument".to_string());
        }
        let weekday = value
            .parse::<i64>()
            .map_err(|_| format!("Invalid weekday value: '{}'", value))?
            as i64;
        if !(0..=6).contains(&weekday) {
            return Err("Weekday modifier accepts values between 0 and 6".to_string());
        }
        return Ok(DateTimeModifier::Weekday(weekday));
    }

    // Parse other modifiers.
    match modifier {
        "ceiling" => return Ok(DateTimeModifier::Ceiling),
        "floor" => return Ok(DateTimeModifier::Floor),
        "start of month" => return Ok(DateTimeModifier::StartOfMonth),
        "start of year" => return Ok(DateTimeModifier::StartOfYear),
        "start of day" => return Ok(DateTimeModifier::StartOfDay),
        "unixepoch" => return Ok(DateTimeModifier::UnixEpoch),
        "julianday" => return Ok(DateTimeModifier::JulianDay),
        "auto" => return Ok(DateTimeModifier::Auto),
        "localtime" => return Ok(DateTimeModifier::Localtime),
        "utc" => return Ok(DateTimeModifier::Utc),
        "subsec" | "subsecond" => return Ok(DateTimeModifier::Subsecond),
        _ => {}
    }

    // At this point we should have either a numeric modifier (1-13 in the SQLite documentation) or an error.
    let original_modifier = modifier;
    let has_sign = modifier.starts_with('+') || modifier.starts_with('-');
    let sign = if modifier.starts_with('-') { -1.0 } else { 1.0 };
    let modifier = modifier.trim_start_matches('+').trim_start_matches('-');

    // Handle modifiers 1-6
    match modifier.split_once(' ').unwrap_or((modifier, "")) {
        (value, "days") => {
            let days = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid days value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(0.0, 0.0, days * sign, 0.0, 0.0, 0.0, 0.0),
            ));
        }
        (value, "hours") => {
            let hours = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid hours value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(0.0, 0.0, 0.0, hours * sign, 0.0, 0.0, 0.0),
            ));
        }
        (value, "minutes") => {
            let minutes = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid minutes value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(0.0, 0.0, 0.0, 0.0, minutes * sign, 0.0, 0.0),
            ));
        }
        (value, "seconds") => {
            let seconds = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid seconds value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(0.0, 0.0, 0.0, 0.0, 0.0, seconds * sign, 0.0),
            ));
        }
        (value, "months") => {
            let months = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid months value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(0.0, months * sign, 0.0, 0.0, 0.0, 0.0, 0.0),
            ));
        }
        (value, "years") => {
            let years = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid years value: '{}'", value))?;
            return Ok(DateTimeModifier::JDNOffset(
                JulianDay::new_relative_from_datetime_vals(years * sign, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            ));
        }
        // At this point all of the numeric modifiers have been parsed. The only remaining ones are 7-13
        (value, "") => {
            if value.contains('-') {
                if !has_sign {
                    return Err(format!("Invalid modifier: '{}'", original_modifier));
                }
                let date = parse_date(value, sign)?;
                return Ok(DateTimeModifier::JDNOffset(date));
            } else {
                let time = parse_time(value, sign)?;
                return Ok(DateTimeModifier::JDNOffset(time));
            }
        }
        (date, time) => {
            if !has_sign {
                return Err(format!("Invalid modifier: '{}'", original_modifier));
            }
            let date = parse_date(date, sign)?;
            let time = parse_time(time, sign)?;
            return Ok(DateTimeModifier::JDNOffset(JulianDay::new(
                date.value() + time.value(),
            )));
        }
    }
}

fn parse_date(date: &str, sign: f64) -> Result<JulianDay, String> {
    if date.is_empty()
        || date.len() != 10
        || date.chars().nth(4) != Some('-')
        || date.chars().nth(7) != Some('-')
    {
        return Err(format!("Invalid date: '{}'.", date));
    }
    let day = date[8..10]
        .parse::<i64>()
        .map_err(|_| format!("Invalid day: '{}'", &date[8..10]))?;
    let year = date[0..4]
        .parse::<i64>()
        .map_err(|_| format!("Invalid year: '{}'", &date[0..4]))?;
    let month = date[5..7]
        .parse::<i64>()
        .map_err(|_| format!("Invalid month: '{}'", &date[5..7]))?;

    if (month != 0 && (month < 1 || month > 12)) || (month == 0 && day != 0) {
        // technically 2025-00-00 is valid
        return Err(format!("Invalid date: '{}'.", date));
    }

    if month != 0 && day != 0 {
        let max_days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        };
        if day < 1 || day > max_days {
            return Err(format!("Invalid date: '{}'.", date));
        }
    }

    Ok(JulianDay::new_relative_from_datetime_vals(
        year as f64 * sign,
        month as f64,
        day as f64,
        0.0,
        0.0,
        0.0,
        0.0,
    ))
}

fn parse_in_range(s: &str, name: &str, min: i64, max: i64) -> Result<i64, String> {
    let value = s
        .parse::<i64>()
        .map_err(|_| format!("Invalid {}: '{}'", name, s))?;
    if !(min..=max).contains(&value) {
        return Err(format!(
            "{} out of range ({}-{}): {}",
            name, min, max, value
        ));
    }
    Ok(value)
}

fn parse_time(time: &str, sign: f64) -> Result<JulianDay, String> {
    if time.is_empty() {
        return Err(format!("Invalid time: '{}'.", time));
    }

    let mut parts = time.split(':');
    let hour = parse_in_range(
        parts
            .next()
            .ok_or_else(|| format!("Invalid time: '{}'.", time))?,
        "hour",
        0,
        23,
    )?;
    let minute = parse_in_range(
        parts
            .next()
            .ok_or_else(|| format!("Invalid time: '{}'.", time))?,
        "minute",
        0,
        59,
    )?;

    let (second, subsecond) = if let Some(second_part) = parts.next() {
        if parts.next().is_some() {
            return Err(format!("Invalid time: '{}'.", time));
        }
        if let Some(dot_pos) = second_part.find('.') {
            if second_part[dot_pos + 1..].len() > 3 {
                return Err(format!("Invalid time: '{}'.", time));
            }
            (
                parse_in_range(&second_part[..dot_pos], "second", 0, 59)?,
                parse_in_range(&second_part[dot_pos + 1..], "subsecond", 0, 999)? as f64 / 1000.0,
            )
        } else {
            (parse_in_range(second_part, "second", 0, 59)?, 0.0)
        }
    } else {
        (0, 0.0)
    };

    Ok(JulianDay::new_relative_from_datetime_vals(
        0.0,
        0.0,
        0.0,
        hour as f64 * sign,
        minute as f64 * sign,
        second as f64 * sign,
        subsecond * sign,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    impl DateTimeModifier {
        fn jdnoffset(&self) -> Option<&JulianDay> {
            match self {
                DateTimeModifier::JDNOffset(jd) => Some(jd),
                _ => None,
            }
        }
    }
    trait ModifierValue {
        fn value(&self) -> f64;
    }

    impl ModifierValue for Result<DateTimeModifier, String> {
        fn value(&self) -> f64 {
            self.as_ref().unwrap().jdnoffset().unwrap().value()
        }
    }

    #[test]
    fn test_parse_modifier() {
        assert!(parse_modifier("5 days").value() == 5.0);
        assert!(parse_modifier("12 hours").value() == 0.5);
        assert!((parse_modifier("30 minutes").value() - 0.020833333333333332).abs() < 0.000001);
        assert!((parse_modifier("45 seconds").value() - 0.0005208333333333333).abs() < 0.000001);
        assert!(parse_modifier("6 months").value() == 183.0);
        assert!(parse_modifier("2 years").value() == 731.0);
        assert!((parse_modifier("12:30").value() - 0.5208333333333333).abs() < 0.000001);
        assert!((parse_modifier("+12:30:45").value() - 0.5213541666666666).abs() < 0.000001);
        assert!((parse_modifier("-12:30:45.123").value() - (-0.5213555903173983)).abs() < 0.000001);
        assert!(parse_modifier("+2025-12-25").value() == 740007.0);
        assert!((parse_modifier("+2025-12-25 12:30").value() - 740007.5208333333).abs() < 0.000001);
        assert!(
            (parse_modifier("+2025-12-25 12:30:45").value() - 740007.5213541666).abs() < 0.000001
        );
        assert!(
            (parse_modifier("+2025-12-25 12:30:45.123").value() - 740007.5213555903).abs()
                < 0.000001
        );
    }

    #[test]
    fn test_parse_modifier_date_requires_sign() {
        assert!(parse_modifier("2025-12-25").is_err());
        assert!(parse_modifier("2025-12-25 12:30").is_err());
        assert!(parse_modifier("2025-12-25 12:30:45").is_err());
        assert!(parse_modifier("2025-12-25 12:30:45.123").is_err());
        assert!(parse_modifier("+2025-12-25").is_ok());
    }

    #[test]
    fn test_parse_modifier_days_without_sign() {
        assert!(parse_modifier("1 days").is_ok());
        assert_eq!(
            parse_modifier("1 days")
                .unwrap()
                .jdnoffset()
                .unwrap()
                .value(),
            1.0
        );
    }
}
