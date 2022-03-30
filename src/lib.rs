#![feature(let_chains)]

use anyhow::{Context, Result};
use email::Email;
use imap::extensions::idle::SetReadTimeout;
use imap::{self};
use std::io::{Read, Write};
pub mod action;
pub mod args;
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
    .native_tls()
    .context("Client builder failed")?;

    let mut imap_session = client
        .login(
            &config
                .connection
                .username,
            &config
                .connection
                .password,
        )
        .map_err(|e| e.0)
        .context("Login failed")?;

    imap_session.select("INBOX")?;

    Ok(imap_session)
}

/// Special behavior: if uid=0, then it fetches the latest message.
/// Note: this assumes 0 is not a valid UID. In practice, this seems
/// to be the case with gmail. In theory, I beleive the specs say that
/// uid could be anything.
pub fn fetch_email(
    uid: u32,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<Email> {
    let uid = if uid == 0 {
        "*".to_string()
    } else {
        uid.to_string()
    };
    let query = "(UID FLAGS INTERNALDATE RFC822 ENVELOPE)";
    let messages = session.uid_fetch(uid, query)?;
    Email::from_fetch(
        messages
            .get(0)
            .context("Empty fetches iterator -- wrong UID?")?,
    )
}

/// Delete a message. Note that no error will be returned if the UID doesn't
/// exist.
pub fn delete(
    uid: u32,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<()> {
    session.uid_store(&uid.to_string(), "+FLAGS (\\DELETED)")?;
    let _deleted = session.uid_expunge(&uid.to_string())?;

    // now we check that a message was actually deleted. There's no error if you
    // call uid_expunge on a non-existant UID, in this context that probably
    // indicates an error. HOWEVER: The reason why this is commented out is
    // because the server has to support QRESYNC in order to get UIDs, which
    // neither greenmail or even gmail supports. I can get the sequence IDs that
    // were deleted, but they're useless to me unless I refactor a bunch of
    // stuff to match UIDs to sequence numbers. ALSO, I  just noticed the "Auto
    // Expunge" setting in Gmail that's turned on by default. It will actually
    // expunge the message as soon as the flag is set, so the client will never
    // get any information back from an expunge response. That leaves me with
    // two options: one, do a fetch and see if it fails; two, have the idle loop
    // actually watch for expunges and try to synchronize that somehow with the
    // other threads... yeah, that sounds like a bad idea.

    // if !deleted
    //     .uids()
    //     .collect::<Vec<u32>>()[..]
    //     .contains(&uid)
    // {
    //     return Err(anyhow!("Failed to delete UID: {}", uid));
    // }

    Ok(())
}

// Move a message. Internally, it's a two-step copy and delete process. Note that
// no error will be returned if you give it a non-existant UID.
pub fn move_email(
    uid: u32,
    mailbox_name: &str,
    session: &mut imap::Session<impl Read + Write + SetReadTimeout>,
) -> Result<()> {
    session.uid_mv(&uid.to_string(), mailbox_name)?;
    delete(uid, session)?;
    Ok(())
}
