use anyhow::Result;
use mail_client::action;
use std::io;
use clap::Parser;
use mail_client::config;


fn main() -> Result<()> {
    let args = Args::parse();
    let config = mail_client::config::get_config(&args.config)?;
    let config = args.overwrite_config(config);

    println!("{:?}", &config);

    let mut session = mail_client::login(&config)?;

    
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    println!("Test");

    let message = action::Message::from_json(&line)?;

    for a in &message.actions {
        match a {
            action::Action::Move(mailbox_name) => {
                mail_client::move_email(message.uid, mailbox_name, &mut session)?
            }
            action::Action::Delete => mail_client::delete(message.uid, &mut session)?,
            action::Action::Label => todo!(),
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