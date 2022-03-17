use anyhow::Result;

use imap::extensions::idle::SetReadTimeout;
use imap::Session;
use lettre;
use lettre::Transport;
use lettre_email;
use mail_client::config;
use native_tls;
use std::io::{Read, Write};

fn get_config() -> config::Config {
    config::Config {
        connection: config::Connection {
            hostname: "127.0.0.1".to_string(),
            username: "readonly-test@greenmail".to_string(),
            password: "readonly-test@greenmail".to_string(),
            port: 3993,
        },
        imap_options: None,
        scripts: None,
    }
}

fn get_session() -> Session<impl Read + Write + SetReadTimeout> {
    let conf = get_config();
    let session = mail_client::login(&conf).unwrap();
    session
}

fn tls() -> native_tls::TlsConnector {
    native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap()
}

fn smtp(user: &str) -> lettre::SmtpTransport {
    let creds = lettre::smtp::authentication::Credentials::new(user.to_string(), user.to_string());
    lettre::SmtpClient::new(
        &format!(
            "{}:3465",
            std::env::var("TEST_HOST").unwrap_or("127.0.0.1".to_string())
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

fn main() -> Result<()> {
    let to = "test@greenmail.com";
    let mut s = smtp(&to);

    let e = lettre_email::Email::builder()
        .from("sender@localhost")
        .to(to)
        .subject("My first e-mail")
        .text("Hello world from SMTP")
        .build()
        .unwrap();
    s.send(e.into())?;

    Ok(())
}
