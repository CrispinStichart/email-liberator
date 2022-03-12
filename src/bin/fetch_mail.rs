use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use clap::Parser;
use imap::extensions::idle::{Handle, SetReadTimeout};
use imap::types::{AttributeValue, UnsolicitedResponse};
use mail_client::config;
use mail_client::email::Email;
use mail_client::login;
use std::io::{Read, Write};

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

fn main() {
    let args = Args::parse();

    let config = config::get_config(args.config);

    if args.catch_up {
        catch_up(&config);
    }

    if !args.no_idle {
        idle(&config);
    }
}

pub fn catch_up(config: &config::Config) {
    if let Some(last_uid) = get_last_message_id() {
        let mut session = login(&config);
        // The '*' means the newest. We add one to the last seen UID
        // so we don't fetch the one we've already seen.
        let range = format!("{}:*", last_uid + 1);
        let query = "FLAGS INTERNALDATE RFC822 ENVELOPE)";
        let messages = session.uid_fetch(range, query);
        if let Ok(messages) = messages {
            for msg in messages.iter() {
                let email = Email::from_fetch(msg);
                output_email(email);
            }
        }
        session
            .logout()
            .unwrap();
    }

    // If there wasn't a UID saved, there's nothing we need to do here.
    ()
}

fn idle(config: &config::Config) {
    let session = Rc::new(RefCell::new(login(&config)));

    let session_copy = Rc::clone(&session);
    session
        .borrow_mut()
        .idle()
        .wait_while(|response| {
            if let UnsolicitedResponse::Fetch { id, attributes } = response {
                output_email(mail_client::fetch_email(id, &mut session_copy.borrow_mut()))
            };
            true
        })
        .unwrap();
}

fn output_email(email: Email) {
    println!(
        "{}",
        email
            .to_json()
            .unwrap()
    )
}

fn get_last_message_id() -> Option<u32> {
    fs::read_to_string(CATCH_UP_FILE)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

fn write_last_message_id(uid: u32) {
    fs::write(CATCH_UP_FILE, uid.to_string()).expect("Couldn't create the catch up file!");
}
