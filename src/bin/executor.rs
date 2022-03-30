use anyhow::Result;
use clap::Parser;
use mail_client::action;
use mail_client::config;
use std::io;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = mail_client::config::get_config(&args.config)?;
    let config = args.overwrite_config(config);

    let mut session = mail_client::login(&config)?;

    loop {
        // Read line from stdin.
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        // Convert the line to a Message, crashing if it can't be parsed.
        let message = action::Message::from_json(&line)?;

        // Labels: rust-imap doesn't support the non-standard IMAP extension Gmail
        // has to support labels (and a few other neat Gmail-specific features).
        // I've opened a ticket on the rust-imap Github page to see what they think
        // of adding it. Note that Gmail exposes the labels as read-only with
        // standard IMAP extension by representing labels as mailboxes, and you can
        // query those mailboxes to see what messages have that label. However,
        // there's no way to assign a label to a message, or directly see what
        // labels a message has, without using their extensions.
        for a in &message.actions {
            match a {
                action::Action::Move(mailbox_name) => {
                    mail_client::move_email(message.uid, mailbox_name, &mut session)?
                }
                action::Action::Delete => mail_client::delete(message.uid, &mut session)?,
                action::Action::Label => todo!(),
            }
        }

        if args
            .forever
            .unwrap_or(false)
        {
            break;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    /// Specify location of config file.
    #[clap(long)]
    pub config: Option<String>,

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

    /// Don't exit after reading first line of stdin.
    #[clap(long)]
    pub forever: Option<bool>,
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
