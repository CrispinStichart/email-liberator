use anyhow::Result;
use serde::Deserialize;
use serde_json;
use std::io;

// this program will accept JSON on stdin that tells it what to do with a message

#[derive(Deserialize, Debug)]
struct Message {
    uid: u32,
    actions: Vec<Action>,
}

#[derive(Deserialize, Debug)]
enum Action {
    Move(String),
    Delete,
    Label,
}

fn main() -> Result<()> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .unwrap();

    let message: Message = serde_json::from_str(&line).unwrap();

    let config = mail_client::config::get_config(None)?;
    let session = mail_client::login(&config)?;

    Ok(())
}
