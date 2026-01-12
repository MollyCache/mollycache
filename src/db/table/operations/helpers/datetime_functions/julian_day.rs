const JULIAN_DAY_NOON_OFFSET: f64 = 0.5;
const UNIX_EPOCH_JULIAN_DAY: f64 = 2440587.5;
const JULIAN_DAY_EPOCH_OFFSET: i64 = 32045;
const YEAR_OFFSET: i64 = 4800;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JulianDay {
    jdn: f64,
}

impl JulianDay {
    pub fn to_calendar_components(&self) -> (i64, i64, i64, i64, i64, i64, f64) {
        let jdn_value = self.value();
        let jd_int = ((jdn_value + JULIAN_DAY_NOON_OFFSET).floor()) as i64;
        let jd_fractional = (jdn_value + JULIAN_DAY_NOON_OFFSET) - (jd_int as f64);

        // Converts Julian Day Number to Gregorian calendar components using inverse calendar-to-JDN formula (see https://en.wikipedia.org/wiki/Julian_day#Converting_Julian_or_Gregorian_calendar_date_to_Julian_Day_Number)
        let day = ((5
            * (((4 * (jd_int + 1401 + (((4 * jd_int + 274277) / 146097) * 3) / 4 - 38) + 3)
                % 1461)
                / 4)
            + 2)
            % 153)
            / 5
            + 1;
        let month = ((5
            * (((4 * (jd_int + 1401 + (((4 * jd_int + 274277) / 146097) * 3) / 4 - 38) + 3)
                % 1461)
                / 4)
            + 2)
            / 153
            + 2)
            % 12
            + 1;
        let year = (4 * (jd_int + 1401 + (((4 * jd_int + 274277) / 146097) * 3) / 4 - 38) + 3)
            / 1461
            - 4716
            + (12 + 2 - month) / 12;

        let total_seconds = (jd_fractional * 86400.0 * 1000.0).round() / 1000.0;
        let hour = (total_seconds / 3600.0).floor() as i64;
        let minute = ((total_seconds % 3600.0) / 60.0).floor() as i64;
        let second_val = (total_seconds % 3600.0) % 60.0;
        let second = second_val.floor() as i64;
        let subsecond = second_val - second as f64;

        (year, month, day, hour, minute, second, subsecond)
    }

    pub fn as_date(&self) -> String {
        let (year, month, day, _, _, _, _) = self.to_calendar_components();
        format!("{:04}-{:02}-{:02}", year, month, day)
    }

    pub fn as_time(&self) -> String {
        let (_, _, _, hour, minute, second, _) = self.to_calendar_components();
        format!("{:02}:{:02}:{:02}", hour, minute, second)
    }

    pub fn as_datetime(&self) -> String {
        let (year, month, day, hour, minute, second, _) = self.to_calendar_components();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }

    pub fn as_unix_epoch(&self) -> f64 {
        (self.jdn - UNIX_EPOCH_JULIAN_DAY) * 86400.0
    }

    pub fn new(jdn: f64) -> Self {
        Self { jdn }
    }

    // https://en.wikipedia.org/wiki/Julian_day
    // This function accepts negative and positive float values for all of the params.
    // If a None is passed in for the year, it will be treated as julian year 0.
    pub fn new_from_datetime_vals(
        year: f64,
        month: f64,
        day: f64,
        hour: f64,
        minute: f64,
        second: f64,
        subsecond: f64,
    ) -> Self {
        let total_seconds = hour * 3600.0 + minute * 60.0 + second + subsecond;
        let time_fraction = total_seconds / 86400.0;

        let year_int = year.floor() as i64;
        let month_int = month.floor() as i64;
        let day_int = day.floor() as i64;

        let a = (14 - month_int) / 12;
        let y = year_int + YEAR_OFFSET - a;
        let m = month_int + 12 * a - 3;

        let jdn_int = day_int + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400
            - JULIAN_DAY_EPOCH_OFFSET;

        let jdn = (jdn_int as f64) + (day - day.floor()) + time_fraction - JULIAN_DAY_NOON_OFFSET;
        Self { jdn }
    }

    pub fn value(&self) -> f64 {
        self.jdn
    }

    pub fn value_mut(&mut self) -> &mut f64 {
        &mut self.jdn
    }
}

pub fn days_in_month(year: i64, month: i64) -> i64 {
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
        _ => unreachable!(),
    }
}

pub fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
