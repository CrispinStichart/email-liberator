use serde::Deserialize;
use std::fs::{self};

const DEFAULT_CONFIG_FILE: &str = "autonomous_mail_client.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub scripts: Option<Vec<Script>>,
}

#[derive(Deserialize, Debug)]
pub struct Script {
    pub interpreter: Option<String>,
    pub location: String,
}

pub fn get_config(file: Option<String>) -> Config {
    let s = fs::read_to_string(file.unwrap_or(DEFAULT_CONFIG_FILE.to_string())).unwrap();
    let config: Config = toml::from_str(&s).unwrap();
    config
}
