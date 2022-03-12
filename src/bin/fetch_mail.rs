use std::fs;

use ::std::sync::{mpsc, Arc, Mutex};
use anyhow::{Context, Result};
use clap::Parser;
use imap::types::UnsolicitedResponse;
use mail_client::config;
use mail_client::email::Email;
use mail_client::login;
use std::thread;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    /// Fetch and process all emails since last run.
    #[clap(long)]
    catch_up: bool,

    /// Disables writing of the UID. Needed if you're running without write priviliages.
    #[clap(long)]
    no_catch_up_write: bool,

    /// Specify location of config file.
    #[clap(long)]
    config: Option<String>,

    /// Specify location of config file.
    #[clap(long)]
    no_idle: bool,
}

const CATCH_UP_FILE: &str = "last_message_id";

fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::get_config(args.config)?;

    if args.catch_up {
        catch_up(&config)?;
    }

    if !args.no_idle {
        idle(config)?
    }

    Ok(())
}

pub fn catch_up(config: &config::Config) -> Result<()> {
    if let Some(last_uid) = get_last_message_id()? {
        let mut session = login(&config)?;
        // The '*' means the newest. We add one to the last seen UID
        // so we don't fetch the one we've already seen.
        let range = format!("{}:*", last_uid + 1);
        let query = "UID FLAGS INTERNALDATE RFC822 ENVELOPE)";
        let messages = session.uid_fetch(range, query);
        let mut last_uid: Option<u32> = None;
        if let Ok(messages) = messages {
            for msg in messages.iter() {
                last_uid = Some(
                    msg.uid
                        .expect("UID wasn't in the fetch query!"),
                );
                let email = Email::from_fetch(msg);
                output_email(email);
            }
        }
        if let Some(uid) = last_uid {
            write_last_message_id(&uid)?;
        }
        session
            .logout()
            .unwrap();
    }

    // If there wasn't a UID saved, there's nothing we need to do here.
    Ok(())
}

fn idle(config: config::Config) -> Result<()> {
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
            .unwrap();
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
        let email = mail_client::fetch_email(&uid, &mut session);
        output_email(email);
    }

    // session.logout()?;

    // Ok(())
}

fn output_email(email: Email) {
    println!(
        "{}",
        email
            .to_json()
            .unwrap()
    )
}

fn get_last_message_id() -> Result<Option<u32>> {
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

fn write_last_message_id(uid: &u32) -> Result<()> {
    Ok(fs::write(CATCH_UP_FILE, uid.to_string()).context("Couldn't create the catch up file!")?)
}
