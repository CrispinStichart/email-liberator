use anyhow::{bail, Result};
use mail_client::action;
use mail_client::config;
use std::io;
use std::process::Command;
use which::which;
fn main() -> Result<()> {
    let scripts = config::get_config(None)?.scripts;
    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        for script in scripts
            .iter()
            .flatten()
        {
            let stop = call_script(script, &line)?;
            if stop {
                break;
            }
        }
    }
}

fn call_script(script: &config::Script, json: &String) -> Result<bool> {
    let mut command = if let Some(interpreter) = &script.interpreter {
        let pb = which(interpreter).unwrap();
        let executable = pb.as_os_str();
        let mut cmd = Command::new(executable);
        cmd.arg(&script.location);
        cmd
    } else {
        Command::new(&script.location)
    };

    command.arg(json);

    let output = command.output()?;

    if output
        .status
        .success()
        && output.stdout.len() > 0
    {
        let message: action::Message =
            action::Message::from_json(std::str::from_utf8(&output.stdout)?)?;

        println!("{}", &message);

        Ok(message
            .stop
            .unwrap_or(false))
    } else {
        bail!("Script returned with non-zero exit code!")
    }
}
