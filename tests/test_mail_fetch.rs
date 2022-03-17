use anyhow::{Context, Result};
use assert_cmd::Command;
use duct::cmd;
use lettre;
use lettre::Transport;
use lettre_email;
use mail_client::binary_libs::fetch_mail_libs;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
mod utils;
use mail_client::email::Email;
use utils::*;

#[test]
fn test_idle() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let cmd = Command::cargo_bin("fetch_mail");

        let output_err = cmd
            .expect("Couldn't find fetch mail program")
            .args(&["--config", "tests/test_config.toml"])
            .timeout(Duration::from_millis(1000))
            .unwrap_err();

        let output = output_err
            .as_output()
            .expect("Couldn't transform error into output");

        let s = String::from_utf8(
            output
                .stdout
                .to_vec(),
        )
        .expect("Couldn't convert string to UTF8");
        tx.send(s).unwrap();
        // give time for parent thread to read from the channel
        sleep(Duration::from_millis(5000));
    });

    sleep(Duration::from_millis(100));

    // let to = random_email();
    let subject = "This test took me 3 hours to write :(";
    send_email(None, None, Some(subject), None)?;

    sleep(Duration::from_millis(100));

    let stdout = rx
        .recv_timeout(Duration::from_secs(10))
        .context("Process did not timeout as expected")?;

    // println!("{}", &stdout);
    let email = Email::from_json(&stdout)?;
    assert_eq!(email.subject, subject);

    Ok(())
}

#[test]
fn test_idle_directly() -> Result<()> {
    let handle = thread::spawn(|| {
        fetch_mail_libs::idle(get_config(None))
            .expect("Something went wrong in the idle testing thread");
    });

    let session = get_session(None)?;
    send_email(None, None, None, None);

    thread::sleep(Duration::from_millis(500));

    Ok(())
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("fetch_mail").unwrap();
    cmd.args(&["--help"]);
    cmd.assert()
        .success();
}
