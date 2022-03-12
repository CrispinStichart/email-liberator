use config::Script;
use email::Email;
use imap::extensions::idle::SetReadTimeout;
use imap::{self};

use std::io;

use std::io::{Read, Write};
use which::which;
pub mod config;
pub mod email;
use anyhow::Result;

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
) -> Email {
    let query = "FLAGS INTERNALDATE RFC822 ENVELOPE)";
    let messages = session.uid_fetch(uid.to_string(), query);

    Email::from_fetch(
        messages
            .unwrap()
            .get(0)
            .unwrap(),
    )
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
