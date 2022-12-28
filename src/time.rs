use chrono::{Datelike, NaiveDateTime, Timelike, Utc};

pub trait RelativeTime {
    /// Return a string describing the relative time between this timestamp and
    /// the current time
    fn relative_time(&self) -> String;
}

impl RelativeTime for NaiveDateTime {
    fn relative_time(&self) -> String {
        let now = Utc::now().naive_utc();
        if self.year() != now.year() {
            format!("{}Y", now.year() - self.year())
        } else if self.month() != now.month() {
            format!("{}M", now.month() - self.month())
        } else if self.day() != now.day() {
            format!("{}D", now.day() - self.day())
        } else if self.hour() != now.hour() {
            format!("{}h", now.hour() - self.hour())
        } else if self.minute() != now.minute() {
            format!("{}m", now.minute() - self.minute())
        } else {
            format!("{}s", now.second() - self.second())
        }
    }
}
