use mail_client::config;
use mail_client::email;
use std::io;
use std::process::{Command, Output};
use which::which;
fn main() {
    let scripts = config::get_config(None)
        .unwrap()
        .scripts;
    loop {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .unwrap();

        for script in scripts
            .iter()
            .flatten()
        {
            call_script(script, &line);
        }
    }
}

fn call_script(script: &config::Script, json: &String) {
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

    let output = command
        .output()
        .unwrap();

    if output
        .status
        .success()
        && output.stdout.len() > 0
    {
        // validate json here? Or leave that up to the program that talks to the server?
        let json_output = std::str::from_utf8(&output.stdout);
        // send json to the output program
    }
}
