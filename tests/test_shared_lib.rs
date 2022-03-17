use anyhow::Result;
use assert_cmd::Command;
use duct::cmd;
use imap::extensions::idle::SetReadTimeout;
use imap::Session;
use lettre;
use lettre::Transport;
use lettre_email;
use mail_client::binary_libs::fetch_mail_libs;
use mail_client::config;
use native_tls;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;

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

#[test]
fn test_login() {
    let mut session = get_session();
    session
        .logout()
        .unwrap();
}

#[test]
fn test_idle() -> Result<()> {
    // let mut cmd = Command::cargo_bin("fetch_mail").unwrap();
    // let location = cmd.get_program();
    // cmd.args(&["--config", "tests/test_config.toml", "--no-idle"]);
    // cmd.assert()
    //     .success();
    // let config = get_config();
    // fetch_mail_libs::idle(config)?;

    let handle = cmd!(r#".\target\debug\fetch_mail.exe"#)
        .stdout_capture()
        .stderr_capture()
        .start()?;

    let d = Duration::from_millis(500);
    sleep(d);

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

    sleep(d);

    handle.kill()?;
    let output = handle.into_output()?;
    println!("{}", std::str::from_utf8(&output.stdout)?);
    // println!("test post");

    Ok(())
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("fetch_mail").unwrap();
    cmd.args(&["--help"]);
    cmd.assert()
        .success();
}
