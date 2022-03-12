use config::Script;
use email::Email;
use imap::extensions::idle::SetReadTimeout;
use imap::{self, Session};
use std::error::Error;
use std::io;

use std::process::{Command, Output};
use std::{
    fs::{self},
    io::{Read, Write},
};
use which::which;
pub mod config;
pub mod email;

pub fn login(config: &config::Config) -> imap::Session<impl Read + Write + SetReadTimeout> {
    let client = imap::ClientBuilder::new(
        &config
            .connection
            .hostname,
        config
            .connection
            .port,
    )
    .native_tls()
    .expect("unable to connect :(");

    let mut imap_session = client
        .login(
            &config
                .connection
                .username,
            &config
                .connection
                .password,
        )
        .map_err(|e| e.0)
        .expect("unable to login :(");

    imap_session
        .select("INBOX")
        .unwrap();

    imap_session
}

pub fn fetch_email(
    uid: u32,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Email {
    let query = "FLAGS INTERNALDATE RFC822 ENVELOPE)";
    let messages = session.uid_fetch(uid.to_string(), query);

    Email::from_fetch(
        messages
            .unwrap()
            .get(0)
            .unwrap(),
    )
}

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

    command.arg(
        message
            .to_json()
            .unwrap(),
    );

    command.output()
}

#[cfg(test)]
mod tests {
    use super::*;

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
