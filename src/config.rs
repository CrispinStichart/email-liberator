use serde::Deserialize;
use std::fs::{self};

const DEFAULT_CONFIG_FILE: &str = "autonomous_mail_client.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub connection: Connection,
    pub imap_options: ImapOptions,
    pub scripts: Option<Vec<Script>>,
}

#[derive(Deserialize, Debug)]
pub struct Connection {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Script {
    pub interpreter: Option<String>,
    pub location: String,
}

#[derive(Deserialize, Debug)]
pub struct ImapOptions {
    sections: Vec<Sections>,
}

#[derive(Deserialize, Debug)]
pub enum Sections {
    FLAGS,
    INTERNALDATE,
    RFC822,
    ENVELOPE,
}

pub fn get_config(file: Option<String>) -> Config {
    let s = fs::read_to_string(file.unwrap_or(DEFAULT_CONFIG_FILE.to_string())).unwrap();
    let config: Config = toml::from_str(&s).unwrap();
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        get_config(None);
    }
}
