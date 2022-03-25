use anyhow::Result;
use clap::Parser;
use mail_client::binary_libs::fetcher_lib::*;
use mail_client::config;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::get_config(&args.config)?;
    let config = args.overwrite_config(config);

    if args.catch_up {
        catch_up(&config, &args)?;
    }

    if !args.no_idle {
        idle(config, &args)?
    }

    thread::sleep(Duration::from_secs(2));
    Ok(())
}
