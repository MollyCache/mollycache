const JULIAN_DAY_NOON_OFFSET: f64 = 0.5;
const UNIX_EPOCH_JULIAN_DAY: f64 = 2440587.5;
const JULIAN_DAY_EPOCH_OFFSET: i64 = 32045;
const YEAR_OFFSET: i64 = 4800;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JulianDay {
    jdn: f64,
    is_subsecond: bool,
}

impl JulianDay {
    pub fn as_date(&self) -> String {
        let jdn_value = self.value();
        let jd_int: i64 = ((jdn_value + JULIAN_DAY_NOON_OFFSET).floor()) as i64;

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
        format!("{:04}-{:02}-{:02}", year, month, day)
    }

    pub fn as_time(&self) -> String {
        let jdn_value = self.value();
        let jd_int = ((jdn_value + JULIAN_DAY_NOON_OFFSET).floor()) as i64;
        let jd_fractional = (jdn_value + JULIAN_DAY_NOON_OFFSET) - (jd_int as f64);
        let total_seconds = jd_fractional * 86400.0;
        let hour = (total_seconds / 3600.0).floor() as i64;
        let minute = ((total_seconds % 3600.0) / 60.0).floor() as i64;
        let second_with_fraction = (total_seconds % 3600.0) % 60.0;
        let second = second_with_fraction.floor() as i64;

        if self.is_subsecond {
            let fractional_seconds = second_with_fraction - second as f64;
            let milliseconds = (fractional_seconds * 1000.0).round() as i64;
            format!(
                "{:02}:{:02}:{:02}.{:03}",
                hour, minute, second, milliseconds
            )
        } else {
            format!("{:02}:{:02}:{:02}", hour, minute, second)
        }
    }

    pub fn as_datetime(&self) -> String {
        format!("{} {}", self.as_date(), self.as_time())
    }

    pub fn as_unix_epoch(&self) -> f64 {
        let jdn_value = self.value();
        if self.is_subsecond {
            (jdn_value - UNIX_EPOCH_JULIAN_DAY) * 86400000.0
        } else {
            (jdn_value - UNIX_EPOCH_JULIAN_DAY) * 86400.0
        }
    }

    pub fn new(jdn: f64) -> Self {
        Self {
            jdn,
            is_subsecond: false,
        }
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
        let day_fraction = day - day.floor();
        let a = (14 - month_int) / 12;
        let y = year_int + YEAR_OFFSET - a;
        let m = month_int + 12 * a - 3;

        let jdn_int = day_int + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400
            - JULIAN_DAY_EPOCH_OFFSET;

        let jdn = (jdn_int as f64) + day_fraction + time_fraction - JULIAN_DAY_NOON_OFFSET;
        Self {
            jdn,
            is_subsecond: false,
        }
    }

    pub fn new_relative_from_datetime_vals(
        y: f64,
        m: f64,
        d: f64,
        h: f64,
        mi: f64,
        s: f64,
        fs: f64,
    ) -> Self {
        let jdn = Self::new_from_datetime_vals(y, m, d, h, mi, s, fs).value();
        let gregorian_year_zero =
            Self::new_from_datetime_vals(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0).value();
        let jdn = jdn - gregorian_year_zero;
        Self {
            jdn,
            is_subsecond: false,
        }
    }

    pub fn value(&self) -> f64 {
        self.jdn
    }

    pub fn value_mut(&mut self) -> &mut f64 {
        &mut self.jdn
    }
}
