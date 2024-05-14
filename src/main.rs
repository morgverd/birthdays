use std::ops::Add;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use chrono::{Datelike, DateTime, Timelike, Utc};
use chrono_tz::Tz;
use crate::birthday::BirthdayPerson;
use crate::config::ConfigFile;

pub mod birthday;
mod config;
mod discord;

pub static BASE_TIMEZONE: Tz = Tz::UTC;
type BirthdayItem = (BirthdayPerson, DateTime<Tz>);

/// Get the current time with base timezone.
fn get_now() -> DateTime<Tz> {
    Utc::now().with_timezone(&BASE_TIMEZONE)
}

/// Get all BirthdayPerson(s) and next birthday DateTime for everyone defined in config.
fn get_birthdays(config: &ConfigFile) -> Vec<BirthdayItem> {

    let now = get_now();
    let mut people = Vec::<BirthdayItem>::with_capacity(config.people.len());

    // Convert each PersonBirthdayConfig into a BirthdayPerson & get the next birthday DateTime (to validate the date).
    for (k, v) in config.people.iter() {
        match BirthdayPerson::from_config(k.clone(), v.clone()) {
            Ok(person) => match person.date.get_next_date(now) {
                None => eprintln!("Couldn't get BirthdayPerson next date for '{k}'!"),
                Some(date) => people.push((person, date))
            },
            Err(e) => eprintln!("Couldn't create BirthdayPerson for '{k}' with error: {e}")
        }
    }
    people
}

/// Only check every hour on the hour.
fn sleep_till_next_hour(now: DateTime<Tz>) -> () {

    // Find the next hour datetime (handling midnight).
    let next_hour = (now.hour() + 1) % 24;
    let dt = if next_hour == 0 {
        match now.date_naive().succ_opt() {
            None => return eprintln!("Could not get next calendar day to sleep till."),
            Some(dt) => match dt.and_hms_opt(0, 0, 0) {
                None => return eprintln!("Could not apply 0 hours to next calendar day NaiveDate."),
                Some(dt) => dt
            }
        }
    } else {
        match now.date_naive().and_hms_opt(next_hour, 0, 0) {
            None => return eprintln!("Could not create next hour NaiveDate."),
            Some(dt) => dt
        }
    };

    // Find duration between next hour date time and now. (Adds 10 seconds for leeway).
    match (dt - now.naive_local()).to_std() {
        Ok(diff) => {
            let duration = diff.add(Duration::from_secs(10));
            println!("Sleeping for {duration:#?} seconds until next hour!");
            sleep(duration)
        },
        Err(_) => eprintln!("Could not convert TimeDelta into std Duration.")
    }
}

fn main() -> () {

    // Read config file.
    let config = match config::read_file() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };

    // Convert config into a set of BirthdayPerson and DateTime(s).
    let mut people = get_birthdays(&config);

    // TODO: REMOVE THIS, FOR DEBUG ONLY.
    for i in 0..3 {
        people.get_mut(i).unwrap().1 = people.get(i).unwrap().1.with_year(2024).unwrap();
    }

    // Main loop.
    let mut first_run = true;
    loop {

        // Sleep until the next hour (unless we're on the first run of loop).
        if !first_run {
            sleep_till_next_hour(get_now());
        } else {
            first_run = false;
        }

        // Get current date / time for testing BirthDates.
        let now = get_now();
        let mut current = Vec::<&BirthdayPerson>::with_capacity(people.len());

        // Find people with a birthday past now and update it.
        for (person, date) in people.iter_mut() {
            if *date < now {
                match person.date.get_next_date(now) {
                    None => eprintln!("Failed to update next birthday DateTime for '{}'", person.name),
                    Some(new_date) => {
                        *date = new_date;
                        current.push(person);
                    }
                }
            }
        }
        if current.is_empty() {
            continue;
        }

        // Now we have everyone whose birthday it currently is, send the discord webhook(s).
        // TODO: Send push notification too via website API.
        discord::run(&config, current);
    }
}
