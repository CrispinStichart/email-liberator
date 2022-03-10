use clap::Parser;

use mail_client::config;
// use config::get_config;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    /// Fetch and process all emails since last run.
    #[clap(long)]
    catch_up: bool,

    /// Disables writing of the UID. Needed if you're running without write priviliages.
    #[clap(long)]
    no_catch_up_write: bool,

    /// Specify location of config file.
    #[clap(long)]
    config: Option<String>,

    /// Specify location of config file.
    #[clap(long)]
    no_idle: bool,
}

fn main() {
    let args = Args::parse();

    let config = config::get_config(args.config);

    if args.catch_up {
        mail_client::catch_up(&config);
    }

    if !args.no_idle {
        mail_client::idle(&config);
    }
}
