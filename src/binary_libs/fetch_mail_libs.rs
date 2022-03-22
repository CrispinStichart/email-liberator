use crate::config;
use crate::email::Email;
use crate::login;
use anyhow::{Context, Result};
use clap::Parser;
use imap::types::UnsolicitedResponse;
use std::fs;
use std::sync::{atomic, mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

// TODO: Move this back into fetch_mail.rs. Will need to add --catch-up-file option for
// testing purposes

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
            output_email(&email);
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
    let (tx, rx) = mpsc::channel();
    let exit_loop = Arc::new(atomic::AtomicBool::new(false));
    let exit_loop_ctrlc_handler = exit_loop.clone();
    let last_seen_uid = Arc::new(Mutex::new(get_last_message_id()?));

    ctrlc::set_handler(move || exit_loop_ctrlc_handler.store(false, atomic::Ordering::Relaxed))?;

    let mut session = login(
        &config
            .lock()
            .unwrap(),
    )?;

    // TODO: Try restoring the idle mechanism and see if I can get code coverage as long
    // as the main thread cleanly terminates.
    let exit_loop_idle_thread = exit_loop.clone();
    let last_seen_uid_idle_thread = last_seen_uid.clone();
    thread::spawn(move || {
        let mut idle_session = login(
            &config
                .lock()
                .unwrap(),
        )
        .expect("Couldn't open session from within idle thread");

        // Check the current last seen
        let mut last_seen = last_seen_uid_idle_thread
            .lock()
            .unwrap();
        // If it's None, that means there was no last_message_id, so we obtain it by
        // fetching * (the most recent message).
        if last_seen.is_none() {
            *last_seen = Some(
                if let Some(most_recent) = idle_session
                    .uid_fetch("*", "UID")
                    .expect("Something went wrong with the fetch")
                    .iter()
                    .next()
                {
                    most_recent
                        .uid
                        .unwrap()
                } else {
                    // In the case of an empty mailbox, we start at zero.
                    0
                },
            )
        }
        // release the mutex lock
        drop(last_seen);

        loop {
            // Check the exit signal, which is set on Ctrl-C/SIGTERM
            if exit_loop_idle_thread.load(atomic::Ordering::Relaxed) {
                break;
            }

            // Construct the UID set -- from the last seen to the newest. We
            // add 1 the last seen so we don't fetch one we've already seen.
            let uid_set = format!(
                "{}:*",
                last_seen_uid_idle_thread
                    .lock()
                    .unwrap()
                    .unwrap()
                    + 1
            );

            // Fetch the entire range.
            let fetches = idle_session
                .uid_fetch(uid_set, "(UID FLAGS INTERNALDATE RFC822 ENVELOPE)")
                .expect("Something went wrong with the fetch");

            for fetch in fetches.iter() {
                // The * operator will always return at least one message. In the common
                // case where there are no new messages, that means the one returned is
                // also the one we saw last, in which case we just skip it.
                if fetch.uid.unwrap()
                    == last_seen_uid_idle_thread
                        .lock()
                        .unwrap()
                        .unwrap_or(0)
                {
                    continue;
                }

                // Finally, we can construct the Email struct and send it to the main thread.
                tx.send(Email::from_fetch(fetch).unwrap())
                    .unwrap();
            }

            // TODO: add timeout to configuration
            let timeout = Duration::from_secs(5);
            thread::sleep(timeout);
        }

        idle_session
            .logout()
            .unwrap();
    });

    // Question: Why not put this in the idle thread?
    // Answer: I don't know T.T
    loop {
        if exit_loop.load(atomic::Ordering::Relaxed) {
            break;
        }

        let timeout = Duration::from_secs(5);
        let email = rx.recv_timeout(timeout);
        if let Ok(email) = email {
            output_email(&email);

            if !args.no_catch_up_write {
                write_last_message_id(email.uid)?;
            }
        }

        thread::sleep(timeout);
    }

    session.logout()?;
    Ok(())
}

pub fn output_email(email: &Email) {
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
