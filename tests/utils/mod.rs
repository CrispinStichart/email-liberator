use anyhow::{anyhow, Context, Result};
use imap::extensions::idle::SetReadTimeout;
use imap::Session;
use lettre;
use lettre::Transport;
use lettre_email;
use mail_client::config;
use native_tls;
use std::io::{Read, Write};
use std::process;

use uuid::Uuid;

pub fn random_email() -> String {
    format!("{}@greenmail.com", Uuid::new_v4()).to_string()
}

pub fn send_email(
    from: Option<&str>,
    to: Option<&str>,
    subject: Option<&str>,
    body: Option<&str>,
) -> Result<()> {
    let to = to.unwrap_or("test@greenmail.com");
    let from = from.unwrap_or("sender@localhost");
    let subject = subject.unwrap_or("test subject");
    let body = body.unwrap_or("Hello world from SMTP");

    let mut s = smtp(&to);
    let e = lettre_email::Email::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .text(body)
        .build()
        .unwrap();
    s.send(e.into())?;
    Ok(())
}

/// When only the recipient matters
pub fn send_email_to(to: &str) -> Result<()> {
    send_email(None, Some(to), None, None)?;
    Ok(())
}

pub fn get_config(to: Option<&str>) -> config::Config {
    let to = to.unwrap_or("test@greenmail.com");
    config::Config {
        connection: config::Connection {
            hostname: "greenmail".to_string(),
            username: to.to_string(),
            password: to.to_string(),
            port: 3993,
        },
        imap_options: None,
        scripts: None,
    }
}

pub fn get_session(to: Option<&str>) -> Result<Session<impl Read + Write + SetReadTimeout>> {
    let conf = get_config(to);
    mail_client::login(&conf)
}

pub fn tls() -> native_tls::TlsConnector {
    native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap()
}

pub fn smtp(user: &str) -> lettre::SmtpTransport {
    let creds = lettre::smtp::authentication::Credentials::new(user.to_string(), user.to_string());
    lettre::SmtpClient::new(
        &format!(
            "{}:3465",
            std::env::var("TEST_HOST").unwrap_or("greenmail".to_string())
        ),
        lettre::ClientSecurity::Wrapper(lettre::ClientTlsParameters {
            connector: tls(),
            domain: "smpt.example.com".to_string(),
        }),
    )
    .unwrap()
    .credentials(creds)
    .transport()
}

/// If there's anything in stderr, we return it as an error. Otherwise, return
/// the stdout as a list of lines.
pub fn parse_output(output: process::Output) -> Result<Vec<String>> {
    if !output
        .status
        .success()
    {
        return Err(anyhow!(
            "Error from catch-up execution:\n{}",
            String::from_utf8(output.stderr)?
        ));
    }

    // otherwise, we turn stdout into a string...
    let stdout = String::from_utf8(
        output
            .stdout
            .to_vec(),
    )
    .context("Couldn't stringify stdout")?;

    // ...and split it on linebreaks.
    let lines = stdout
        .to_owned()
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Ok(lines)
}
