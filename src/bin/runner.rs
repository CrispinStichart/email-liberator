use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use config::EmailField;
use mail_client::action;
use mail_client::config;
use mail_client::email;
use std::io;
use std::process::Command;
use which::which;

// TODO: option to limit the input to the script to one field, for example
//       just the body, or just the header. Useful for integrating with
//       external tools without needing a wrapper script.
fn main() -> Result<()> {
    let args = Args::parse();
    let config = mail_client::config::get_config(&args.config)?;
    let config = args.overwrite_config(config);

    let scripts = config.scripts;

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        for script in scripts
            .iter()
            .flatten()
        {
            // Scripts can use stop if they do something like delete
            // an email that will cause scripts later in the pipeline
            // to fail.
            let output = call_script(
                script,
                &line,
                script
                    .email_field
                    .as_ref(),
            )?;
            if let Some(msg_str) = output {
                let stop = output_message(&msg_str)?;
                if stop {
                    break;
                }
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

/// Call an external program and return the stdout wrapped in Ok(), or
/// the stderr wrapped in an Err() if the program exits with a non-zero
/// exit code.
fn call_script(
    script: &config::Script,
    json: &str,
    email_field: Option<&EmailField>,
) -> Result<Option<String>> {
    let email = email::Email::from_json(json)?;

    let cmd_input = match email_field {
        Some(email_field) => match email_field {
            EmailField::ADDRESS => todo!(),
            EmailField::SUBJECT => email.subject,
            EmailField::BODY => email.body,
            EmailField::UID => email
                .uid
                .to_string(),
        },

        None => json.to_string(),
    };

    let mut command = if let Some(interpreter) = &script.interpreter {
        let pb = which(interpreter).unwrap();
        let executable = pb.as_os_str();
        let mut cmd = Command::new(executable);
        cmd.arg(&script.location);
        cmd
    } else {
        Command::new(&script.location)
    };

    command.arg(cmd_input);

    let output = command.output()?;

    if output
        .status
        .success()
    {
        if output
            .stdout
            .is_empty()
        {
            Ok(None)
        } else {
            Ok(Some(String::from_utf8(output.stdout)?))
        }
    } else {
        return Err(anyhow!("Script returned with non-zero exit code!")
            .context(String::from_utf8(output.stderr)?));
    }
}

/// Convert the JSON string into a message object, output the JSON again on stdout,
/// and return the `stop` paramater to indicate whether the email should be
/// processed by future scripts.
fn output_message(message_str: &str) -> Result<bool> {
    let message: action::Message = action::Message::from_json(message_str)?;

    println!("{}", &message);

    Ok(message
        .stop
        .unwrap_or(false))
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
