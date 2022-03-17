use std::fs;

use anyhow::Result;
use clap::Parser;
use mail_client::binary_libs::fetch_mail_libs::*;
use mail_client::config;

fn main() -> Result<()> {
    let args = Args::parse();

    let config = config::get_config(args.config)?;

    if args.catch_up {
        catch_up(&config)?;
    }

    if !args.no_idle {
        println!("Now idling");
        idle(config)?
    }

    Ok(())
}
