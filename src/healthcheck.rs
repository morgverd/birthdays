use std::thread;
use std::time::Duration;
use crate::config::HealthcheckConfig;

pub fn start(config: &HealthcheckConfig) -> () {

    let timeout = Duration::from_secs(config.interval);
    let url = config.url.clone();

    thread::spawn(move || {
        println!("Started healthcheck ping thread!");

        loop {
            if let Err(e) = minreq::get(&url).send() {
                eprintln!("Could not send healthcheck request with error: {}", e);
            }
            thread::sleep(timeout)
        }
    });
}