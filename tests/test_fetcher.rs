use anyhow::Result;
use assert_cmd;
use mail_client::binary_libs::fetcher_lib;
use mail_client::email::Email;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Child;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub mod utils;
use utils::*;

const TIMEOUT_SECS: u64 = 10;

#[cfg(target_os = "linux")]
use libc;

/// Attempts to kill the child gracefully. This is mainly for the infinite loops
/// in fetcher. If we hard-kill the child, we don't get code coverage.
#[cfg(target_os = "linux")]
pub fn kill_child(child: &mut Child) -> Result<()> {
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    sleep(Duration::from_secs(TIMEOUT_SECS));
    if child
        .try_wait()
        .is_err()
    {
        child.kill()?
    }
    Ok(())
}

/// Windows doesn't have signals.
#[cfg(not(target_os = "linux"))]
pub fn kill_child(child: &mut Child) -> Result<()> {
    child.kill()?;
    Ok(())
}

#[test]
fn test_idle() -> Result<()> {
    let username = random_email();

    // I started writing this on windows. D:
    let program = if cfg!(unix) {
        "./target/debug/fetcher"
    } else {
        ".\\target\\debug\\fetcher.exe"
    };

    // Launch the program in an entirely seperate process.
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

    // Grab an active pipe to the process's stdout.
    let stdout = child
        .stdout
        .take()
        .unwrap();

    // Use a thread to read stdout and read a line.
    let (tx, rx) = mpsc::channel();
    // We have to use an Arc to ensure that tx isn't dropped when the
    // thread finishes, because then the recv will fail.
    let tx = Arc::new(Mutex::new(tx));
    let tx2 = tx.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut buf = String::new();
        reader
            .read_line(&mut buf)
            .unwrap();
        tx2.lock()
            .unwrap()
            .send(buf)
            .unwrap();
    });

    // This was an attempt at delaying the sending of an email until process was
    // up and running. However: recent tests have show that it takes a little
    // under a second before the process actually gets to the .idle() call,
    // which means 100ms isn't enough. HOWEVER, it's still working correctly,
    // so... I don't know. Maybe greenmail takes even longer than that to
    // recieve an email?
    // sleep(Duration::from_millis(100));

    // True story.
    let subject = "This test took me 3 hours to write :(";
    send_email(None, Some(&username), Some(subject), None)?;

    // Here's why we used a thread to read stdout -- we want to be able to time
    // out if we don't get a response from the client.
    let stdout = rx.recv_timeout(Duration::from_secs(TIMEOUT_SECS))?;

    let email = Email::from_json(&stdout)?;
    assert_eq!(email.subject, subject);

    // Attempt to gracefully kill the child. Important for getting accurate code
    // coverage; LLVM can't record coverage if the program crashes.
    kill_child(&mut child)?;

    Ok(())
}

// Run program with the --catch-up argument and return the output.
fn run_catch_up(email: &str) -> Result<Vec<String>> {
    // Spawn the process and wait for output.
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

    parse_output(output)
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
