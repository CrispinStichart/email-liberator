use anyhow::Result;
mod utils;
use utils::*;

#[test]
fn test_login() -> Result<()> {
    let mut session = get_session(None)?;
    session
        .logout()
        .unwrap();
    Ok(())
}

#[test]
fn test_fetch() -> Result<()> {
    let to = random_email();
    let subject = Some("My first e-mail");
    send_email(None, to.as_str().into(), subject, None)?;

    let mut session = get_session(to.as_str().into())?;
    let mail = mail_client::fetch_email(&1, &mut session)?;

    assert_eq!(subject.unwrap(), mail.subject);

    Ok(())
}

#[test]
fn test_delete() -> Result<()> {
    let to = random_email();

    let mut session = get_session(to.as_str().into())?;
    let mailbox = session.select("INBOX")?;
    assert_eq!(0, mailbox.exists);
    send_email(None, Some(&to), None, None)?;
    let mailbox = session.select("INBOX")?;

    assert_eq!(1, mailbox.exists);

    let _mail = mail_client::delete(&1, &mut session)?;
    let mailbox = session.select("INBOX")?;

    assert_eq!(0, mailbox.exists);

    Ok(())
}
