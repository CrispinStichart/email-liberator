use anyhow::Result;
use serde::Deserialize;
use std::fs::{self};

const DEFAULT_CONFIG_FILE: &str = "autonomous_mail_client.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub connection: Connection,
    pub imap_options: Option<ImapOptions>,
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
    pub sortkey: Option<i32>,
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

// TODO: Overwrite configuration with command-line arguments
pub fn get_config(file: &Option<String>) -> Result<Config> {
    let s = fs::read_to_string(
        file.as_ref()
            .unwrap_or(&DEFAULT_CONFIG_FILE.to_string()),
    )?;
    let config: Config = toml::from_str(&s)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        get_config(&None).unwrap();
    }
}
