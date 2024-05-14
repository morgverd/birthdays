use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PersonDiscordConfig {
    pub id: Option<String>,
    pub servers: Vec<String>,
    #[serde(default)] pub ping_everyone: Option<bool>
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersonBirthdayConfig {
    pub date: (u32, u32),
    pub tz: String,
    #[serde(default)] pub discord: Option<PersonDiscordConfig>
}

#[derive(Debug, Deserialize)]
pub struct DiscordServerConfig {
    pub webhook: String,
    #[serde(default)] pub default_ping_everyone: bool
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub people: HashMap<String, PersonBirthdayConfig>,
    pub servers: HashMap<String, DiscordServerConfig>
}

pub fn read_file() -> Result<ConfigFile, String> {

    // Open config file.
    let mut file = match File::open("birthdays.json") {
        Ok(file) => file,
        Err(_) => return Err("Could not open JSON config file!".to_owned())
    };

    // Read the file content into a string.
    let mut json_content = String::new();
    if let Err(_) = file.read_to_string(&mut json_content) {
        return Err("Failed to read JSON config file!".to_owned());
    }

    // Deserialize JSON.
    match serde_json::from_str::<ConfigFile>(&json_content) {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("Couldn't deserialize ConfigFile JSON: {}", e.to_string()).to_owned())
    }
}