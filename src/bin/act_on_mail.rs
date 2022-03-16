use anyhow::Result;
use serde::Deserialize;
use serde_json;
use std::io;

use mail_client::action;
fn main() -> Result<()> {
    let config = mail_client::config::get_config(None)?;
    let mut session = mail_client::login(&config)?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    let message = action::Message::from_json(&line)?;

    for a in &message.actions {
        match a {
            action::Action::Move(mailbox_name) => {
                session.uid_mv(
                    &message
                        .uid
                        .to_string(),
                    mailbox_name,
                )?;
            }
            action::Action::Delete => mail_client::delete(&message.uid, &mut session)?,
            action::Action::Label => todo!(),
        }
    }

    Ok(())
}
