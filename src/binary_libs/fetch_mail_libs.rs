use std::fs;

use crate::config;
use crate::email::Email;
use crate::login;
use anyhow::{Context, Result};
use clap::Parser;
use imap::types::UnsolicitedResponse;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    /// Fetch and process all emails since last run.
    #[clap(long)]
    pub catch_up: bool,

    /// Disables writing of the UID. Needed if you're running without write priviliages.
    #[clap(long)]
    pub no_catch_up_write: bool,

    /// Specify location of config file.
    #[clap(long)]
    pub config: Option<String>,

    /// Don't enter the idle loop
    #[clap(long)]
    pub no_idle: bool,

    /// hostname of IMAP server.
    #[clap(long)]
    pub hostname: Option<String>,

    /// port of IMAP server.
    #[clap(long)]
    pub port: Option<u16>,

    /// username for IMAP authentication
    #[clap(long)]
    pub username: Option<String>,

    /// password for IMAP authentication.
    #[clap(long)]
    pub password: Option<String>,
}

// Note: https://docs.rs/merge/latest/merge/ exists. Can we use that, plus
// maybe a custom trait, to merge args and config?
impl Args {
    #[rustfmt::skip]
    pub fn overwrite_config(&self, config: config::Config) -> config::Config {
        config::Config {
            connection: config::Connection {
                hostname : self.hostname.as_ref().unwrap_or(&config.connection.hostname).clone(),
                username : self.username.as_ref().unwrap_or(&config.connection.username).clone(),
                password : self.password.as_ref().unwrap_or(&config.connection.password).clone(),
                port : self.port.unwrap_or(config.connection.port),
            },
            ..config    
        }
    }
}

pub const CATCH_UP_FILE: &str = "last_message_id";

pub fn catch_up(config: &config::Config, args: &Args) -> Result<()> {
    if let Some(last_uid) = get_last_message_id()? {
        let mut session = login(config)?;
        // The '*' means the newest. We add one to the last seen UID
        // so we don't fetch the one we've already seen. However, *
        // will ALWAYS return at least one result, so we handle that
        // later.
        let range = format!("{}:*", last_uid + 1);
        let query = "(UID FLAGS INTERNALDATE RFC822 ENVELOPE)";
        let messages = session.uid_fetch(range, query)?;
        let mut new_last_uid: Option<u32> = None;
        for msg in messages.iter() {
            new_last_uid = Some(
                msg.uid
                    .context("UID wasn't in the fetch query!")?,
            );
            // We skip the message if we saw it already.
            if new_last_uid.unwrap() == last_uid {
                continue;
            }
            
            let email = Email::from_fetch(msg)?;
            output_email(email);
        }
        if let Some(uid) = new_last_uid {
            if !&args.no_catch_up_write {
                write_last_message_id(uid)?;
            }
        }
        session
            .logout()
            .unwrap();
    }

    // If there wasn't a UID saved, there's nothing we need to do here.
    Ok(())
}

pub fn idle(config: config::Config, args: &Args) -> Result<()> {
    let config = Arc::new(Mutex::new(config));
    let (tx, rx) = mpsc::channel::<u32>();

    let mut session = login(
        &config
            .lock()
            .unwrap(),
    )?;

    thread::spawn(move || {
        let mut idle_session = login(
            &config
                .lock()
                .unwrap(),
        )
        .expect("Couldn't open session from within idle thread");

        idle_session
            .idle()
            .wait_while(|response| {
                // The server sends Exists when the number of messages changes.
                // TODO: 1: Try to not do something when the count drops, because that
                //          will happen if, durther down the pipeline, a filter moves or
                //          deletes something. Check the current count when we start
                //          idling, update it as we get responses, and only
                //          fetch and send email when the count goes up?
                if let UnsolicitedResponse::Exists(count) = response {
                    tx.send(count)
                        .unwrap();
                }
                true
            })
            .expect("Something happened during the idle wait while");
    });

    loop {
        // We're not doing anything with the count right now, just using it's
        // existance as a signal.
        let _count = rx.recv().unwrap();

        // We use the "*" operator to fetch the newest message. This assumes that
        // the server always sends an EXISTS for each message, and doesn't batch them.
        // Also it assumes that a second message didn't arive in the fraction of a
        // second it takes to do this operatioon.
        let uid = session
            .uid_fetch("*", "UID")?
            .iter()
            .next()
            .context("Looks like the mailbox was empty?")?
            .uid
            .context("Fetch response didn't contain UID")?;
        let email = crate::fetch_email(uid, &mut session)?;
        output_email(email);

        if !args.no_catch_up_write {
            write_last_message_id(uid)?;
        }
    }

    // Unreachable, lol
    // session.logout()?;
    // Ok(())
}

pub fn output_email(email: Email) {
    println!(
        "{}",
        email
            .to_json()
            .unwrap()
    )
}

pub fn get_last_message_id() -> Result<Option<u32>> {
    let read_result = fs::read_to_string(CATCH_UP_FILE);

    if let Err(e) = read_result {
        if e.kind() == std::io::ErrorKind::NotFound {
            return Ok(None);
        }
        return Err(e.into());
    };

    let parse_results = read_result
        .unwrap()
        .trim()
        .parse::<u32>();

    match parse_results {
        Ok(uid) => Ok(Some(uid)),
        Err(e) => Err(e)
            .context("The message ID file contained something that wasn't a parsable integer."),
    }
}

pub fn write_last_message_id(uid: u32) -> Result<()> {
    Ok(fs::write(CATCH_UP_FILE, uid.to_string()).context("Couldn't create the catch up file!")?)
}
