[![codecov](https://codecov.io/gh/CrispinStichart/email-liberator/branch/main/graph/badge.svg?token=TTIIJOTXNY)](https://codecov.io/gh/CrispinStichart/email-liberator)

# Email Liberator

# Current State: WORK IN PROGRESS

This project is a simple email client, intended to be run 24/7 that:

1. watches for incoming mail
2. sends text of messages to user-defined programs or scripts
3. optionally receives handling instructions back from those programs to move, delete, or tag messages

## Use Cases

* advanced filtering beyond what your mail provider offers
  * the reason I built this tool is so that I can filter my emails based on arbitrary regex. Gmail supports custom filters, but these are limited to using Gmail's search syntax, which isn't very sophisticated.
* backups
  * you could even have a script to automatically print a hardcopy of every email
* data collection
* custom notifications

## Configuration

Configuration will be through a TOML file. This file will provide server information (hostname, username, password) as well as a list of each program to feed emails to. Any option in the TOML file can be supplied on the command line instead.

Development note: still trying to figure out how to do that without copy and pasting things. I wanna do it *right*, goddammit. I'm thinking a custom macro could work...

## Real-Time or Polling

There is a `--catch-up` flag that, if set, will look for a `last_message_id` file, which contains the ID of the last message downloaded by the client. If it exists, the client will fetch all messages with a higher ID. (Side note: the IMAP protocol does not have the capability to sort by a timestamp any more accurate than the date, hence the usage of the ID.)

Regardless of whether the `last_message_id` previously existed, the client will then write the ID of most recently downloaded message, before going into the IDLE state. In the IDLE state, it will continually update `last_message_id` as new emails come in.

If `--no-idle` is set, the client will instead exit after the catch-up step. In this way, you can configure the client to run periodically, via a `cron` job or other scheduling service, if you don't need to take action in real time.

## Do one thing, and do it well

This project is actually comprised of three binaries, that can be chained together with pipes, UNIX style.

* `fetcher` will idle (or be run in catch up mode, see above), and emit all messages on `stdout` in JSON
* `runner` will accept an email via `stdin` and run it through all the scripts in the in the configuration file
  * It will check for a response and then pass the response and UID to a third binary
* `executor` will accept response information, including the email's UID, from `stdin`, and then talk to the webserver and take the action specified

They will all share the configuration file, but they can also all be used independently in any sort of pipeline you want. For example, you might only use `fetcher` to archive emails, or if you only have one script to run you might omit `runner` and pipe `fetcher` straight to your script.

### Actions Supported

Using the `executor` program, you can delete or move a message. Other actions can be added in the future. Unfortunately, Gmail labels use a non-standard extension to the IMAP protocol that the library I'm using, `rust-imap`, doesn't support. I've taken a look at the code, and it may be within my abilities to add that feature.

# Testing

Since there isn't a lot of "library" code in this project, unit tests aren't much help. Luckily, we can use Greenmail for integration testing. Greenmail is a mail server built for integration testing. Getting Greenmail to work with SSL/TLS in an integration testing environment is a bit tricky; I have [instructions on my blog](https://crispinstichart.github.io/using-SSL-in-greenmail-docker-container/).

We have code coverage provided by the LLVM source-based coverage tooling in Rust's nightly branch. See `.github/workflows/build-and-test.yml` for details. I ended up making two custom docker containers, which was a simple solution, but not strictly necessary.

# Current Status:

The core functions are written and tested.

`fetcher` is pretty much done but I keep rewriting the IDLE loop, just to make it work with the code coverage tool (code coverage isn't generated on a crash, but it's hard to cleanly exit from the IDLE loop). Also there seems to be some random crashes in the casual user testing I've done, so that's not great.

`runner` is a skeleton. It's also boring, so I'm leaving it for last.

`executor` is also a skeleton, but really doesn't need much more. I need to do some tests with regards to labels.

The configuration poses an interesting problem. I want to be able to specify all settings in either the TOML file or on the command line. However, only some of the options apply to all the binaries.

Both the CLI arguments and configuration file options are specified in the code as structs. I use `clap` to turn the argument struct into a CLI interface, and the configuration struct is used by `serde` to deserialize the TOML file.  What this comes down to is figuring out how to merge structs of different types.

I *know* I can just copy and paste a tiny amount of code into a mere three different files, but... it feels bad.

My current plan is to learn more about how macros work. From what I understand right now, I should be able to write a macro that takes two structs as arguments and spits out a combined one.

Update: did a bit of research, and what I wanted to do is not possible. My desire was to be able to do something like this:

```rust
struct ArgOne {
  foo: String
}

struct ArgTwo {
  bar: u32
}

merge_structs!("Combined", ArgOne, ArgTwo);
```

and the macro call would generate:

```rust
struct Combined {
  foo: String,
  bar: u32
}
```

However, it turns out that a macro cannot actually inspect a struct. It can only recognize `ArgOne` and `ArgTwo` as types, and can't go any further.

Update update: Okay, so applies to `macro_rules!` defined macros. But there are also *procedural* macros, which are different and have more power. They run during compilation, rather than during... tokenization, I think is the other step?

`derive` macros are procedural.

Okay. Ugh. After some more reading, procedural macros won't work either. Using a derive or attribute macro, I can inspect one struct. However, I can't get a reference to another struct while doing that.

The next option is external code generation. I feel like this is a bit of a code smell, but hear me out.

I'll write a complete settings TOML file, then parse it to generate the rust code I want.

a realistic but minimal example:

```python
[connection]

username = ""
password = ""

[fetcher]

# Don't enter idle loop
no_idle = false

[executor]

# Read from stdin forever, rather than just one line
looping = false
```

This will be parsed to generate rust code that looks like this:

```rust
#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct FetcherArgs {
  #[clap(long)]
  pub username: Option<String>,

  #[clap(long)]
  pub password: Option<String>,

  /// Don't enter idle loop
  #[clap(long)]
  pub no_idle: bool
}

impl FetcherArgs {
  pub fn merge_config(&self, config: Config) -> Config {
    config::Config {
      hostname : self.hostname.as_ref().unwrap_or(&config.connection.hostname).clone(),
      username : self.username.as_ref().unwrap_or(&config.connection.username).clone(),
      no_idle : self.port.unwrap_or(config.no_idle),
      ..config
    }
  }
}

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct ExecutorArgs {
  #[clap(long)]
  pub username: Option<String>,

  #[clap(long)]
  pub password: Option<String>,

  /// Read from stdin forever, rather than just one line
  #[clap(long)]
  pub looping: bool
}

impl ExecutorArgs {
  pub fn merge_config(&self, config: Config) -> Config {
    config::Config {
      hostname : self.hostname.as_ref().unwrap_or(&config.connection.hostname).clone(),
      username : self.username.as_ref().unwrap_or(&config.connection.username).clone(),
      looping : self.port.unwrap_or(config.looping),
      ..config
    }
  }
}

pub struct Config {
  pub username: String,
  pub password: String,
  pub no_idle: bool,
  pub looping: bool
}

impl struct Config {
  pub fn new() -> Self {
    Config {
      username: "",
      password: "",
      no_idle: false,
      looping: false
    }
  }
}
```

Look at all that bullshit I won't have to write by hand. Of course, perhaps needing to write all that bullshit is a sign I'm doing something dumb.

I could use sub-commands instead, and pass around the Args struct directly rather than a config struct. I'd have to put everything back into one binary, though...

If I did this the code would look like

```rust
#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct ExecutorArgs {
  #[clap(long)]
  pub username: Option<String>,

  #[clap(long)]
  pub password: Option<String>,

  #[clap(subcommand)]
  pub command: Commands
}

#[derive(Subcommand)]
enum Commands {
  Fetcher {
    /// Don't enter idle loop
    #[clap(long)]
    pub no_idle: bool
  },
  Executor {
    /// Read from stdin forever, rather than just one line
    #[clap(long)]
    pub looping: bool
  }
}
```

Another option is to forgo type checking, and store configuration in a hashmap indexed by strings. I'm not a big fan of that. Having these strings be provided by a bunch of `const`s in a module would make it slightly better. I would still have multiple sources of truth, however.


I took a look at how cargo does it. In `command_prelude.rs`, they have a common set of arguments defined as a `clap` command, and then the individual binaries call the `subcommand` function, which returns the command builder, and they extend it that way. So I was wrong in my assumption that using subcommands would require one binary. And if fact, this pattern doesn't seem to even require use of subcommands.

Two things about this: One, it uses the builder pattern and not the derive pattern like I'm using. Which I could switch to, absolutely. Two, it still doesn't address the main issue I have, which is sharing information between the config file and the arguments.