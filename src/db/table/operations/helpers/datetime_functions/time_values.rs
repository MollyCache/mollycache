use crate::db::table::core::value::Value;

const UNIX_EPOCH_JULIAN_DAY: f64 = 2440587.5;
const MILLISECONDS_PER_DAY: f64 = 86400000.0;
const JULIAN_DAY_EPOCH_OFFSET: i64 = 32045;
const JULIAN_DAY_NOON_OFFSET: f64 = 0.5;
const YEAR_OFFSET: i64 = 4800;

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// Parses a string to an i64 within a given range, i.e. parse hour and validate within 0-23.
fn parse_in_range(s: &str, name: &str, min: i64, max: i64, default: i64) -> Result<i64, String> {
    if s.is_empty() {
        return Ok(default);
    }
    let value = s
        .parse::<i64>()
        .map_err(|_| format!("Invalid {}: {:?}", name, s))?;
    if !(min..=max).contains(&value) {
        return Err(format!(
            "{} out of range ({}-{}): {}",
            name, min, max, value
        ));
    }
    Ok(value)
}

// https://en.wikipedia.org/wiki/Julian_day
fn calculate_julian_day(y: i64, m: i64, d: i64, h: i64, mi: i64, s: i64, fs: f64) -> f64 {
    let a = (14 - m) / 12;
    let y = y + YEAR_OFFSET - a;
    let m = m + 12 * a - 3;

    let jdn_int =
        d + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - JULIAN_DAY_EPOCH_OFFSET;

    let time_fraction = (h as f64) / 24.0 + (mi as f64) / 1440.0 + (s as f64 + fs) / 86400.0;

    (jdn_int as f64) + time_fraction - JULIAN_DAY_NOON_OFFSET
}

// This is parsed according to the SQLite documentation for Time Values
// https://sqlite.org/lang_datefunc.html
// This function takes a time value and returns the corresponding f64 julian day number
pub fn parse_timevalue(time_value: &Value) -> Result<f64, String> {
    match time_value {
        Value::Text(text) if text == "now" => {
            let duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("System time error: {}", e))?;
            let unix_ms = duration.as_secs() as i64 * 1000 + duration.subsec_millis() as i64;
            // Convert to Julian Day Number
            let jdn = (unix_ms as f64 / MILLISECONDS_PER_DAY) + UNIX_EPOCH_JULIAN_DAY;
            Ok(jdn)
        }
        Value::Text(txt) => {
            // Look at formats 1-10 in the SQLite documentation for Time Values
            if txt.is_empty() || txt.starts_with(char::is_whitespace) {
                return Err(format!("Invalid time value: {:?}", time_value)); // empty or leading whitespace is invalid
            }
            let txt = txt.trim_end();

            let (txt, timezone_part) = if txt.ends_with('Z') {
                (&txt[..txt.len() - 1], "") // Timezones are already UTC / Zulu time
            } else if txt.len() >= 6 {
                // Check if the last 6 chars are [+/-]HH:MM
                let last_six = &txt[txt.len() - 6..];
                if (last_six.starts_with('+') || last_six.starts_with('-'))
                    && last_six.chars().nth(3) == Some(':')
                    && last_six[1..3].chars().all(|c| c.is_ascii_digit())
                    && last_six[4..6].chars().all(|c| c.is_ascii_digit())
                {
                    (&txt[..txt.len() - 6], last_six)
                } else {
                    (txt, "")
                }
            } else {
                (txt, "")
            };

            // We first check for a ' ' or a T to split on otherwise if it contains a '-' we assume it's a date only.
            let (date_part, time_part) = txt
                .split_once(' ')
                .or_else(|| txt.split_once('T'))
                .unwrap_or_else(|| {
                    if txt.contains('-') {
                        (txt, "")
                    } else {
                        ("", txt)
                    }
                });

            let (year, rest) = date_part.split_once('-').unwrap_or(("2000", date_part));
            let (month, day) = rest.split_once('-').unwrap_or(("01", rest));
            let day = if day.is_empty() { "01" } else { day };

            let (hour, rest) = time_part.split_once(':').unwrap_or(("00", time_part));
            let (minute, rest) = rest.split_once(':').unwrap_or((rest, ""));
            let (second, millisecond) = if rest.is_empty() {
                ("00", "0")
            } else {
                rest.split_once('.').unwrap_or((rest, "0"))
            };

            let year = parse_in_range(year, "year", 0, 9999, 2000)?;
            let month = parse_in_range(month, "month", 1, 12, 1)?;
            let day = parse_in_range(day, "day", 1, 31, 1)?;

            let max_days = days_in_month(year, month);
            if day > max_days {
                return Err(format!(
                    "day out of range for {}-{:02}: {} (max: {})",
                    year, month, day, max_days
                ));
            }

            let hour = parse_in_range(hour, "hour", 0, 23, 0)?;
            let minute = parse_in_range(minute, "minute", 0, 59, 0)?;
            let second = parse_in_range(second, "second", 0, 59, 0)?;

            let millisecond = if millisecond.len() > 3 {
                &millisecond[..3]
            } else {
                millisecond
            };
            let frac_seconds = if !millisecond.is_empty() {
                let frac_str = if millisecond.len() > 9 {
                    &millisecond[..9]
                } else {
                    millisecond
                };
                let frac_val = frac_str.parse::<f64>().unwrap_or(0.0);
                frac_val / 10_f64.powi(frac_str.len() as i32)
            } else {
                0.0
            };

            let mut jdn =
                calculate_julian_day(year, month, day, hour, minute, second, frac_seconds);

            // timezone adjustment
            let (timezone_hour, timezone_minute) = if timezone_part.len() == 6 {
                (&timezone_part[1..3], &timezone_part[4..6])
            } else {
                ("00", "00")
            };

            if !timezone_part.is_empty() {
                let tz_hour = parse_in_range(timezone_hour, "timezone hour", 0, 14, 0)? as f64; // Timezone hours are lmtd to 0-14
                let tz_minute =
                    parse_in_range(timezone_minute, "timezone minute", 0, 59, 0)? as f64;
                let tz_offset = tz_hour / 24.0 + tz_minute / 1440.0;

                if timezone_part.starts_with('-') {
                    jdn += tz_offset;
                } else if timezone_part.starts_with('+') {
                    jdn -= tz_offset;
                }
            }

            Ok(jdn)
        }
        Value::Integer(jdn_int) => Ok(*jdn_int as f64),
        Value::Real(jdn_float) => Ok(*jdn_float),
        _ => Err(format!("Invalid time value: {:?}", time_value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timevalue() {
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12 12:00:00".to_string())),
            Ok(2461022.0)
        );

        assert_eq!(parse_timevalue(&Value::Real(2461021.5)), Ok(2461021.5));
        assert_eq!(parse_timevalue(&Value::Integer(2461021)), Ok(2461021.0));
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12".to_string())),
            Ok(2461021.5)
        );
        let result = parse_timevalue(&Value::Text("2025-12-12 12:30".to_string())).unwrap();
        assert!((result - 2461022.020833333).abs() < 0.000001);
        let result = parse_timevalue(&Value::Text("2025-12-12 12:00:00.123".to_string())).unwrap();
        assert!((result - 2461022.0000014235).abs() < 0.0000001);
        let result = parse_timevalue(&Value::Text("2025-12-12T12:30".to_string())).unwrap();
        assert!((result - 2461022.020833333).abs() < 0.000001);
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12T12:00:00".to_string())),
            Ok(2461022.0)
        );
        assert_eq!(
            parse_timevalue(&Value::Text("12:00:00".to_string())),
            Ok(2451545.0)
        );
        let result = parse_timevalue(&Value::Text("12:30".to_string())).unwrap();
        assert!((result - 2451545.020833333).abs() < 0.000001);
        assert_eq!(
            parse_timevalue(&Value::Text("0000-01-01".to_string())),
            Ok(1721059.5)
        );
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12 12:00:00.1234567890".to_string())),
            Ok(2461022.0000014235)
        );

        // Timezone tests
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12T12:00:00Z".to_string())),
            Ok(2461022.0)
        );
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12 12:00:00Z".to_string())),
            Ok(2461022.0)
        );
        let result =
            parse_timevalue(&Value::Text("2025-12-12T12:00:00-04:00".to_string())).unwrap();
        assert!((result - 2461022.1666666665).abs() < 0.0000001);
        let result =
            parse_timevalue(&Value::Text("2025-12-12T12:00:00+04:00".to_string())).unwrap();
        assert!((result - 2461021.8333333335).abs() < 0.0000001);
        assert_eq!(
            parse_timevalue(&Value::Text("12:00:00Z".to_string())),
            Ok(2451545.0)
        );
        let result = parse_timevalue(&Value::Text("08:00:00-04:00".to_string())).unwrap();
        assert!((result - 2451545.0).abs() < 0.0000001);
        let result = parse_timevalue(&Value::Text("16:00:00+04:00".to_string())).unwrap();
        assert!((result - 2451545.0).abs() < 0.0000001);

        let now_result = parse_timevalue(&Value::Text("now".to_string())).unwrap();
        assert!(now_result > 2460000.0 && now_result < 2470000.0);

        // Trailing whitespace is ignored
        assert_eq!(
            parse_timevalue(&Value::Text("2025-12-12 12:00:00    ".to_string())),
            Ok(2461022.0)
        );
        // Leading whitespace should fail
        assert!(parse_timevalue(&Value::Text("   2025-12-12 12:00:00".to_string())).is_err());

        let result = parse_timevalue(&Value::Text("12:00:00.500".to_string())).unwrap();
        assert!((result - 2451545.0000057870).abs() < 0.0000001);

        let result = parse_timevalue(&Value::Text("2025-12-12 12:00:00.1".to_string())).unwrap();
        assert!((result - 2461022.0000011574).abs() < 0.0000001);
        let result = parse_timevalue(&Value::Text("2025-12-12 12:00:00.12".to_string())).unwrap();
        assert!((result - 2461022.0000013889).abs() < 0.0000001);

        // Negative Julian day numbers
        assert_eq!(parse_timevalue(&Value::Integer(-1)), Ok(-1.0));
        assert_eq!(parse_timevalue(&Value::Real(-100.5)), Ok(-100.5));
    }

    #[test]
    fn test_parse_timevalue_invalid_values() {
        assert!(parse_timevalue(&Value::Null).is_err());
        assert!(parse_timevalue(&Value::Blob(vec![1, 2, 3])).is_err());

        assert!(parse_timevalue(&Value::Text("".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-99-12 12:00:00Z".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-99 12:00:00".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-12 99:00:00".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-12 12:60:00".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-12 12:00:99".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-12 12:00:00-99:00".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025-12-12 12:00:00+00:99".to_string())).is_err());

        assert!(parse_timevalue(&Value::Text("10000-01-01".to_string())).is_err());

        assert!(parse_timevalue(&Value::Text("not a date".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("hello world".to_string())).is_err());
        assert!(parse_timevalue(&Value::Text("2025/12/12".to_string())).is_err()); // wrong separator
        assert!(parse_timevalue(&Value::Text("   ".to_string())).is_err());
    }
}
