use anyhow::{anyhow, Result};
use email::Email;
use imap::extensions::idle::SetReadTimeout;
use imap::{self};

use std::io::{Read, Write};
pub mod action;
pub mod binary_libs;
pub mod config;
pub mod email;

// TODO: add option to open mailbox in read-only (with .examine() instead of .select())
pub fn login(config: &config::Config) -> Result<imap::Session<impl Read + Write + SetReadTimeout>> {
    let client = imap::ClientBuilder::new(
        &config
            .connection
            .hostname,
        config
            .connection
            .port,
    )
    .native_tls()?;

    let mut imap_session = client
        .login(
            &config
                .connection
                .username,
            &config
                .connection
                .password,
        )
        .map_err(|e| e.0)?;

    imap_session.select("INBOX")?;

    Ok(imap_session)
}

pub fn fetch_email(
    uid: &u32,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<Email> {
    let query = "(FLAGS INTERNALDATE RFC822 ENVELOPE)";
    let messages = session.uid_fetch(uid.to_string(), query)?;

    Email::from_fetch(
        messages
            .get(0)
            .expect("Failed to get first message from messages"),
    )
}

pub fn delete(
    uid: &u32,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<()> {
    session.uid_store(&uid.to_string(), imap::types::Flag::Deleted.to_string())?;
    session.uid_expunge(&uid.to_string())?;
    Ok(())
}

pub fn move_email(
    uid: &u32,
    mailbox_name: &str,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<()> {
    session.uid_mv(&uid.to_string(), mailbox_name)?;
    delete(uid, session)?;
    Ok(())
}
