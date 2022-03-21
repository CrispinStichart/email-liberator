use anyhow::{anyhow, Result};
use assert_cmd::Command;
use mail_client::action;
pub mod utils;
use utils::*;

fn run_act_on_mail(email: &str, input: &str) -> Result<Vec<String>> {
    let cmd = Command::cargo_bin("act_on_mail");
    let output = cmd
        .expect("Couldn't find fetch mail program")
        .args(&[
            "--config",
            "tests/test_config.toml",
            "--username",
            format!("{}", &email).as_str(),
            "--password",
            format!("{}", &email).as_str(),
        ])
        .write_stdin(input.to_string() + "\n")
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
fn test_act_on_mail() -> Result<()> {
    let delete_second_email = action::Message {
        uid: 2,
        actions: vec![action::Action::Delete],
        stop: None,
    }
    .to_string();

    let to_email = random_email();
    utils::send_email_to(&to_email)?;
    send_email_to(&to_email)?;
    send_email_to(&to_email)?;

    run_act_on_mail(&to_email, &delete_second_email)?;

    let mut session = get_session(Some(&to_email))?;

    assert!(mail_client::fetch_email(1, &mut session).is_ok());
    assert!(mail_client::fetch_email(2, &mut session).is_err());
    assert!(mail_client::fetch_email(3, &mut session).is_ok());

    Ok(())
}
