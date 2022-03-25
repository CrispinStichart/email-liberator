// use crate::config;
// use anyhow::{Context, Result};
// use clap::Parser;

// /// Rationale: we want certain shared arguments for the binaries. Specifically,
// /// the connection arguments. However, we also want to specify them in a config file,
// /// and we want extra, binary specific options that can be in the arguments or the
// /// config file. Command line arguments should overwrite all config file settings.
// ///
// /// It would also be really nice to be able to generate an example config file
// /// from the clap-derived struct. Not sure how that would work.
// ///
// /// Ways I can do this:
// ///
// /// 1. Copy and paste, like I'm doing now. Pros: it works. Cons: ugly, harder to update.
// /// 2. Use escaped positionals (--). We'd get the common args, then the specific ones after
// ///    the --. Pros: kind of like how cargo does it. Cons: not sure if clap would generate
// ///    the help page for the extra commands.
// /// 3. Use a macro. Pros: I get to learn macros. Cons: it might not work.
// ///
// /// Macro ideas: inline! macro that accepts a struct as an argument, then copy and pastes
// /// the macro fields. Problem: Macro can't appear as a struct field, I think. Could also
// /// have a combine! macro that accepts multiple structs and merges them.

// #[derive(Parser, Debug)]
// #[clap(author, version)]
// pub struct CommonArgs {
//     /// hostname of IMAP server.
//     #[clap(long)]
//     pub hostname: Option<String>,

//     /// port of IMAP server.
//     #[clap(long)]
//     pub port: Option<u16>,

//     /// username for IMAP authentication
//     #[clap(long)]
//     pub username: Option<String>,

//     /// password for IMAP authentication.
//     #[clap(long)]
//     pub password: Option<String>,
// }

// // Note: https://docs.rs/merge/latest/merge/ exists. Can we use that, plus
// // maybe a custom trait, to merge args and config?
// impl CommonArgs {
//     #[rustfmt::skip]
//     pub fn overwrite_config(&self, config: config::Config) -> config::Config {
//         config::Config {
//             connection: config::Connection {
//                 hostname : self.hostname.as_ref().unwrap_or(&config.connection.hostname).clone(),
//                 username : self.username.as_ref().unwrap_or(&config.connection.username).clone(),
//                 password : self.password.as_ref().unwrap_or(&config.connection.password).clone(),
//                 port : self.port.unwrap_or(config.connection.port),
//             },
//             ..config
//         }
//     }
// }
