use anyhow::Result;
use assert_cmd::Command;
use mail_client::action;
pub mod utils;
use utils::*;

fn run_act_on_mail(email: &str, input: &str) -> Result<Vec<String>> {
    // Run the process and wait for output
    let cmd = Command::cargo_bin("executor");
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

    parse_output(output)
}

#[test]
fn test_delete() -> Result<()> {
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

#[test]
fn test_move() -> Result<()> {
    let move_email = action::Message {
        uid: 1,
        actions: vec![action::Action::Move("SPAM".to_owned())],
        stop: None,
    }
    .to_string();

    let to_email = random_email();

    // send the email
    send_email_to(&to_email)?;

    let mut session = get_session(Some(&to_email))?;
    // create the SPAM mailbox
    session.create("SPAM")?;

    // make sure it's there
    assert!(mail_client::fetch_email(1, &mut session).is_ok());

    // issue the move command
    run_act_on_mail(&to_email, &move_email)?;

    // shouldn't be there any more
    assert!(mail_client::fetch_email(1, &mut session).is_err());

    // should be in the SPAM mailbox
    session.select("SPAM")?;
    assert!(mail_client::fetch_email(1, &mut session).is_ok());

    Ok(())
}
