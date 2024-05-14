use std::str::FromStr;
use chrono::{Datelike, DateTime, MappedLocalTime, TimeZone};
use chrono_tz::Tz;
use crate::config::{PersonBirthdayConfig, PersonDiscordConfig};

#[derive(Debug)]
pub struct BirthdayPerson {
    pub name: String,
    pub date: Birthdate,
    pub discord: Option<PersonDiscordConfig>
}
impl BirthdayPerson {
    pub fn from_config(name: String, config: PersonBirthdayConfig) -> Result<Self, String> {

        let date = match Birthdate::from_config(&config) {
            Ok(date) => date,
            Err(error) => return Err(format!("Failed to create BirthDate: {error}").to_owned())
        };

        Ok(BirthdayPerson {
            name,
            date,
            discord: config.discord
        })
    }
}

#[derive(Debug)]
pub struct Birthdate {
    day: u32,
    month: u32,
    tz: Tz
}
impl Birthdate {

    fn from_config(config: &PersonBirthdayConfig) -> Result<Self, &'static str> {

        // Convert timezone from config string.
        let timezone = match Tz::from_str(config.tz.as_str()) {
            Ok(timezone) => timezone,
            Err(_) => return Err("Could not convert timezone!")
        };

        Ok(Birthdate {
            day: config.date.0,
            month: config.date.1,
            tz: timezone
        })
    }

    pub fn get_next_date(&self, now: DateTime<Tz>) -> Option<DateTime<Tz>> {

        // Create a date with a year (UTC localised).
        let create_date = |y| -> Option<DateTime<Tz>> {
            match self.tz.with_ymd_and_hms(
                y, self.month, self.day,
                0, 0, 0
            ) {
                MappedLocalTime::Single(dt) => Some(dt),
                MappedLocalTime::Ambiguous(_, dt) => Some(dt),
                MappedLocalTime::None => None
            }
        };

        // Create date with current year.
        let current_year = now.year();
        let mut next_date = match create_date(current_year) {
            None => return None,
            Some(dt) => dt
        };

        // If the next date is before current date, increment year.
        if next_date < now {
            next_date = match create_date(current_year + 1) {
                None => return None,
                Some(dt) => dt
            }
        }
        Some(next_date)
    }
}
