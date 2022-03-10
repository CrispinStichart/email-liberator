use config::Script;
use imap::{self};
use serde::{Deserialize, Serialize};
use std::io;

use std::process::{Command, Output};
use std::{
    fs::{self},
    io::{Read, Write},
};
use which::which;

pub mod config;

const CATCH_UP_FILE: &str = "last_message_id";

#[derive(Serialize, Deserialize)]
struct Email {
    sender: String,
    subject: String,
    body: String,
}

impl Email {
    fn from_fetch(msg: &imap::types::Fetch) -> Email {
        Email {
            // jesus christ :|
            // This looks gnarly as heck; there's got to be a better way.
            // Is there a macro that can unwrap multiple nested items?
            sender: msg
                .envelope()
                .unwrap()
                .sender
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| format!("{:?}", a))
                .collect::<Vec<String>>()
                .join(" "),
            subject: msg
                .envelope()
                .unwrap()
                .subject
                .as_ref()
                .and_then(|cow| std::str::from_utf8(cow).ok())
                .unwrap()
                .to_string(),
            body: "not implemented yet".to_owned(),
            // body: std::str::from_utf8(msg.body().unwrap())
            //     .unwrap()
            //     .to_string(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

fn login(config: &config::Config) -> imap::Session<impl Read + Write> {
    let client = imap::ClientBuilder::new(&config.hostname, config.port)
        .native_tls()
        .expect("unable to connect :(");

    let mut imap_session = client
        .login(&config.username, &config.password)
        .map_err(|e| e.0)
        .expect("unable to login :(");

    imap_session
        .select("INBOX")
        .unwrap();

    imap_session
}

pub fn catch_up(config: &config::Config) {
    if let Some(last_uid) = get_last_message_id() {
        let mut session = login(&config);
        // The '*' means the newest. We add one to the last seen UID
        // so we don't fetch the one we've already seen.
        let range = format!("{}:*", last_uid + 1);
        let query = "FLAGS INTERNALDATE RFC822 ENVELOPE)";
        let messages = session.uid_fetch(range, query);
        if let Ok(messages) = messages {
            for msg in messages.iter() {
                let email = Email::from_fetch(msg);
                run_all_scripts(email, config)
            }
        }
    }

    // If there wasn't a UID saved, there's nothing we need to do here.
    ()
}

pub fn idle(_config: &config::Config) {}

fn run_all_scripts(message: Email, config: &config::Config) {
    for script in config
        .scripts
        .iter()
        .flatten()
    {
        run_script(script, &message).unwrap();
    }
}

fn run_script(script: &Script, message: &Email) -> io::Result<Output> {
    let mut command = if let Some(interpreter) = &script.interpreter {
        let pb = which(interpreter).unwrap();
        let executable = pb.as_os_str();
        let mut cmd = Command::new(executable);
        cmd.arg(&script.location);
        cmd
    } else {
        Command::new(&script.location)
    };

    command.arg(message.to_json());

    command.output()
}

fn get_last_message_id() -> Option<u32> {
    fs::read_to_string(CATCH_UP_FILE)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

fn write_last_message_id(uid: u32) {
    fs::write(CATCH_UP_FILE, uid.to_string()).expect("Couldn't create the catch up file!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_config;

    #[test]
    fn test_run_script() {
        let s = Script {
            interpreter: Some("python3.9".to_string()),
            location: "tests/test_echo.py".to_string(),
        };
        let email = Email {
            sender: "test sender".to_owned(),
            subject: "test subject".to_owned(),
            body: "test body".to_owned(),
        };
        let out = run_script(&s, &email).unwrap();
        let outs = std::str::from_utf8(&out.stdout).unwrap();
        assert_eq!(outs, "test body");
    }
}
