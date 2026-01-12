#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeModifier {
    AddYears(f64),
    AddMonths(f64),
    AddDays(f64),
    AddHours(f64),
    AddMinutes(f64),
    AddSeconds(f64),
    ShiftDate {
        years: f64,
        months: f64,
        days: f64,
    },
    ShiftTime {
        hours: f64,
        minutes: f64,
        seconds: f64,
    },
    ShiftDateTime {
        years: f64,
        months: f64,
        days: f64,
        hours: f64,
        minutes: f64,
        seconds: f64,
    },
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
        (value, "day") | (value, "days") => {
            let days = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid days value: '{}'", value))?;
            return Ok(DateTimeModifier::AddDays(days * sign));
        }
        (value, "hour") | (value, "hours") => {
            let hours = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid hours value: '{}'", value))?;
            return Ok(DateTimeModifier::AddHours(hours * sign));
        }
        (value, "minute") | (value, "minutes") => {
            let minutes = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid minutes value: '{}'", value))?;
            return Ok(DateTimeModifier::AddMinutes(minutes * sign));
        }
        (value, "second") | (value, "seconds") => {
            let seconds = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid seconds value: '{}'", value))?;
            return Ok(DateTimeModifier::AddSeconds(seconds * sign));
        }
        (value, "month") | (value, "months") => {
            let months = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid months value: '{}'", value))?;
            return Ok(DateTimeModifier::AddMonths(months * sign));
        }
        (value, "year") | (value, "years") => {
            let years = value
                .parse::<f64>()
                .map_err(|_| format!("Invalid years value: '{}'", value))?;
            return Ok(DateTimeModifier::AddYears(years * sign));
        }
        // At this point all of the numeric modifiers have been parsed. The only remaining ones are 7-13
        (value, "") => {
            if value.contains('-') {
                if !has_sign {
                    return Err(format!("Invalid modifier: '{}'", original_modifier));
                }
                let (years, months, days) = parse_date_shift(value, sign)?;
                return Ok(DateTimeModifier::ShiftDate {
                    years,
                    months,
                    days,
                });
            } else {
                let (hours, minutes, seconds) = parse_time_shift(value, sign)?;
                return Ok(DateTimeModifier::ShiftTime {
                    hours,
                    minutes,
                    seconds,
                });
            }
        }
        (date, time) => {
            if !has_sign {
                return Err(format!("Invalid modifier: '{}'", original_modifier));
            }
            // For composite "+YYYY-MM-DD HH:MM:SS", we need to return something that applies both.
            // But our structure is one modifier.
            // We can return a ShiftDate, but we need to signal that there is also time.
            // Wait, SQLite treats these as separate modifiers effectively?
            // "The modifier can also be of the form ±YYYY-MM-DD HH:MM:SS"
            // Let's verify if we can split this.
            // Actually, we can just error out here or handle it.
            // If we split it, we'd need to return multiple modifiers, but signature is single.
            // Let's make ShiftDate also capable of shifting time? Or just use a composite struct?
            // Actually, let's just make `ShiftDate` have optional time?
            // Or `ShiftDateTime`.
            // Let's check `parse_modifier` usage. It's called in a loop.
            // If we encounter this, we can't return two.
            // But wait, the loop in `mod.rs` splits by arguments.
            // `date('...', '+1 year')`. `+1 year` is one arg.
            // `date('...', '+1 year 2 months')` -> this is not valid in SQLite as one string?
            // SQLite modifiers are separate arguments usually?
            // `date('now', '+1 year', '+1 month')`.
            // BUT `date('now', '+1 year +1 month')` is NOT valid.
            // However, `+YYYY-MM-DD HH:MM:SS` IS valid as a SINGLE modifier string.
            // So we need a `ShiftDateTime` variant.

            let (years, months, days) = parse_date_shift(date, sign)?;
            let (hours, minutes, seconds) = parse_time_shift(time, sign)?;
            // For simplicity, let's just use ShiftDate and ShiftTime if we could, but we can't return list.
            // Let's add ShiftDateTime.
            return Ok(DateTimeModifier::ShiftDateTime {
                years,
                months,
                days,
                hours,
                minutes,
                seconds,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateShift {
    pub years: f64,
    pub months: f64,
    pub days: f64,
}

fn parse_date_shift(date: &str, sign: f64) -> Result<(f64, f64, f64), String> {
    if date.len() != 10 || date.chars().nth(4) != Some('-') || date.chars().nth(7) != Some('-') {
        return Err(format!("Invalid date format in modifier: '{}'", date));
    }
    let year = date[0..4]
        .parse::<f64>()
        .map_err(|_| format!("Invalid year: '{}'", &date[0..4]))?;
    let month = date[5..7]
        .parse::<f64>()
        .map_err(|_| format!("Invalid month: '{}'", &date[5..7]))?;
    let day = date[8..10]
        .parse::<f64>()
        .map_err(|_| format!("Invalid day: '{}'", &date[8..10]))?;

    Ok((year * sign, month * sign, day * sign))
}

fn parse_time_shift(time: &str, sign: f64) -> Result<(f64, f64, f64), String> {
    let mut parts = time.split(':');
    let hour = parts
        .next()
        .ok_or_else(|| format!("Invalid time: '{}'", time))?
        .parse::<f64>()
        .map_err(|_| format!("Invalid hour: '{}'", time))?;
    let minute = parts
        .next()
        .ok_or_else(|| format!("Invalid time: '{}'", time))?
        .parse::<f64>()
        .map_err(|_| format!("Invalid minute: '{}'", time))?;

    let (second, subsecond) = if let Some(second_part) = parts.next() {
        if let Some(dot_pos) = second_part.find('.') {
            (
                second_part[..dot_pos]
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid second: '{}'", time))?,
                second_part[dot_pos + 1..]
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid subsecond: '{}'", time))?
                    / 10f64.powi((second_part.len() - dot_pos - 1) as i32),
            )
        } else {
            (
                second_part
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid second: '{}'", time))?,
                0.0,
            )
        }
    } else {
        (0.0, 0.0)
    };

    Ok((hour * sign, minute * sign, (second + subsecond) * sign))
}

impl DateTimeModifier {
    // Helper to allow extending the enum in `mod.rs` without large changes if we had used ShiftDateTime
    // We added ShiftDateTime to the enum above so we are good.
    // We need to update the enum definition in the code block above to include ShiftDateTime
}

// Re-defining enum to include ShiftDateTime properly
// (This is just for my own thought process, the file write will be correct)
