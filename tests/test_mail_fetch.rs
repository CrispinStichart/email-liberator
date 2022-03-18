use anyhow::{anyhow, Context, Result};
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
    let username = random_email();
    let thread_username = username.clone();
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let cmd = Command::cargo_bin("fetch_mail");

        let output_err = cmd
            .expect("Couldn't find fetch mail program")
            .args(&[
                "--config",
                "tests/test_config.toml",
                "--no-catch-up-write",
                "--username",
                &thread_username,
                "--password",
                &thread_username,
            ])
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
        sleep(Duration::from_millis(1000));
    });

    sleep(Duration::from_millis(100));

    // let to = random_email();
    let subject = "This test took me 3 hours to write :(";
    send_email(None, Some(&username), Some(subject), None)?;

    let stdout = rx
        .recv_timeout(Duration::from_secs(10))
        .context("Process did not timeout as expected")?;

    handle
        .join()
        .expect("Idle testing thread failed to join");

    let email = Email::from_json(&stdout)?;
    assert_eq!(email.subject, subject);

    Ok(())
}

fn run_catch_up(email: &str) -> Result<Vec<String>> {
    let cmd = Command::cargo_bin("fetch_mail");

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
    fetch_mail_libs::write_last_message_id(0)?;

    send_email(None, Some(&email), None, None)?;

    sleep(Duration::from_millis(500));
    assert_eq!(1, run_catch_up(&email)?.len());

    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;

    sleep(Duration::from_millis(500));
    assert_eq!(3, run_catch_up(&email)?.len());

    send_email(None, Some(&email), None, None)?;
    send_email(None, Some(&email), None, None)?;

    sleep(Duration::from_millis(500));
    assert_eq!(2, run_catch_up(&email)?.len());

    // now it shouldn't see any because it's all caught up
    assert_eq!(0, run_catch_up(&email)?.len());

    Ok(())
}
// The issue with this test is that we can't capture the output.
// #[test]
// fn test_idle_directly() -> Result<()> {
//     let to = random_email();
//     let opt_to = Some(to.as_str());
//     let thread_to = to.clone();
//     let handle = thread::spawn(move || {
//         fetch_mail_libs::idle(get_config(Some(&thread_to)), &get_args())
//             .expect("Something went wrong in the idle testing thread");
//     });

//     let session = get_session(opt_to)?;
//     send_email(None, opt_to, None, None);

//     thread::sleep(Duration::from_millis(500));

//     Ok(())
// }

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("fetch_mail").unwrap();
    cmd.args(&["--help"]);
    cmd.assert()
        .success();
}

#[test]
fn test_get_last_message_id() -> Result<()> {
    fetch_mail_libs::write_last_message_id(0)?;
    let id = fetch_mail_libs::get_last_message_id()?;
    assert_eq!(0, id.unwrap());

    fetch_mail_libs::write_last_message_id(42)?;
    let id = fetch_mail_libs::get_last_message_id()?;
    assert_eq!(42, id.unwrap());
    Ok(())
}
