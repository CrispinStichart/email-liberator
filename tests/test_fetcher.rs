use anyhow::{anyhow, Result};
use assert_cmd;
use mail_client::binary_libs::fetcher_lib;
use mail_client::email::Email;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Child;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub mod utils;
use utils::*;

const TIMEOUT_SECS: u64 = 10;

#[cfg(target_os = "unix")]
use libc;

/// Attempts to kill the child gracefully. This is mainly
/// for the infinite loops in fetcher. If we hard-kill
/// the child, we don't get code coverage.
#[cfg(target_os = "unix")]
pub fn kill_child(child: &mut Child) -> Result<()> {
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    sleep(Duration::from_secs(TIMEOUT_SECS));
    if child
        .try_wait()
        .is_none()
    {
        child.kill()?
    }
    Ok(())
}

#[cfg(not(target_os = "unix"))]
pub fn kill_child(child: &mut Child) -> Result<()> {
    child.kill()?;
    Ok(())
}

#[test]
fn test_idle() -> Result<()> {
    let username = random_email();

    let program = if cfg!(unix) {
        "./target/debug/fetcher"
    } else {
        ".\\target\\debug\\fetcher.exe"
    };
    let mut cmd = Command::new(program);

    let mut child = cmd
        .stdout(Stdio::piped())
        .args(&[
            "--config",
            "tests/test_config.toml",
            "--no-catch-up-write",
            "--username",
            &username,
            "--password",
            &username,
        ])
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .unwrap();

    let mut reader = BufReader::new(stdout);
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut buf = String::new();
        reader
            .read_line(&mut buf)
            .unwrap();
        tx.send(buf)
            .unwrap();
    });

    sleep(Duration::from_millis(100));

    let subject = "This test took me 3 hours to write :(";
    send_email(None, Some(&username), Some(subject), None)?;

    let stdout = rx.recv_timeout(Duration::from_secs(TIMEOUT_SECS))?;

    let email = Email::from_json(&stdout)?;
    assert_eq!(email.subject, subject);

    kill_child(&mut child)?;

    Ok(())
}

fn run_catch_up(email: &str) -> Result<Vec<String>> {
    let cmd = assert_cmd::Command::cargo_bin("fetcher");

    let output = cmd
        .expect("Couldn't find fetch mail program")
        .args(&[
            "--config",
            "tests/test_config.toml",
            "--no-idle",
            "--catch-up",
            "--username",
            format!("{}", &email).as_str(),
            "--password",
            format!("{}", &email).as_str(),
        ])
        .output()?;

    if output.stderr.len() > 0 {
        return Err(anyhow!(
            "Error from catch-up execution:\n{}",
            String::from_utf8(output.stderr)?
        ));
    }

    let stdout = String::from_utf8(
        output
            .stdout
            .to_vec(),
    )?;

    let lines = stdout
        .to_owned()
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Ok(lines)
}

#[test]
fn test_catchup() -> Result<()> {
    let email = random_email();
    // write the ID file. UIDs start at 1, so fetching from zero will get
    // everything in the mailbox.

    fetcher_lib::write_last_message_id(0)?;
    // should be nothing to start out with
    assert_eq!(0, run_catch_up(&email)?.len());

    send_email(None, Some(&email), None, None)?;

    assert_eq!(1, run_catch_up(&email)?.len());

    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;

    assert_eq!(3, run_catch_up(&email)?.len());

    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;

    assert_eq!(2, run_catch_up(&email)?.len());

    // now it shouldn't see any because it's all caught up
    assert_eq!(0, run_catch_up(&email)?.len());

    Ok(())
}

#[test]
fn test_help() {
    let mut cmd = assert_cmd::Command::cargo_bin("fetcher").unwrap();
    cmd.args(&["--help"]);
    cmd.assert()
        .success();
}

#[test]
fn test_get_last_message_id() -> Result<()> {
    fetcher_lib::write_last_message_id(0)?;
    let id = fetcher_lib::get_last_message_id()?;
    assert_eq!(0, id.unwrap());

    fetcher_lib::write_last_message_id(42)?;
    let id = fetcher_lib::get_last_message_id()?;
    assert_eq!(42, id.unwrap());
    Ok(())
}
