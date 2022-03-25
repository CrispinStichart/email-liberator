use anyhow::Result;
use mail_client::config;

pub fn get_config() -> config::Config {
    config::Config {
        connection: config::Connection {
            hostname: "127.0.0.1".to_string(),
            username: "test@greenmail.com".to_string(),
            password: "test@greenmail.com".to_string(),
            port: 3993,
        },
        imap_options: None,
        scripts: None,
    }
}
fn main() -> Result<()> {
    let mut session = mail_client::login(&get_config())?;

    for c in session
        .capabilities()?
        .iter()
    {
        println!("{:?}", c);
    }

    let fetches = session.fetch("0:*", "(UID FLAGS INTERNALDATE ENVELOPE)")?;

    for f in fetches.iter() {
        println!("{:?}", f);
    }

    Ok(())
}
