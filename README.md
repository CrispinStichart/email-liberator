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

Configuration will be through a TOML file. This file will provide server information (hostname, username, password) as well as a list of each program to feed emails to.

Each program can be customized to receive different segments of the message (e.g. sender, subject line, body, or some combination thereof) in different ways. The configuration will also specify what and how the program will communicate back to the client. 

**Communication Methods**

* JSON via `stdin` -- development priority
* file saved to temp directory
  * standard email format -- mbox? Binary?
    * needs research
* socket maybe?

Communication in reverse will also be accepted via those channels.

## Real-Time or Polling

There is a `--catch-up` flag that, if set, will look for a `last_message_id` file, which contains the ID of the last message downloaded by the client. If it exists, the client will fetch all messages with a higher ID. (Side note: the IMAP protocol does not have the capability to sort by a timestamp any more accurate than the date, hence the usage of the ID.)

Regardless of whether the `last_message_id` previously existed, the client will then write the ID of most recently downloaded message, before going into the IDLE state. In the IDLE state, it will continually update `last_message_id` as new emails come in.

If `--no-idle` is set, the client will instead exit after the catch-up step. In this way, you can configure the client to run periodically, via a `cron` job or other scheduling service, if you don't need to take action in real time. 

## Do one thing, and do it well?

I've just been struck by the inspiration to ship this as multiple binaries, Unix style. 

* One will idle, and emit all messages on stdout in JSON (this one could also do the catch up)
* One will accept an email via stdin and run it through all the scripts in the in the configuration file
  * It will check for a response and then pass the response and UID to a third binary
* One will accept response information, including the email's UID, from stdin, and then talk to the webserver.

They will all share the configuration file, but they can also all be used independently in any sort of pipeline you want. 

With this in mind, it makes sense to allow the idle/catch-up program to be configured to output whatever IMAP-protocol fields you want. 

# Testing

Since there isn't a lot of "library" code in this project, unit tests aren't much help. Luckily, we can use Greenmail for integration testing. Greenmail is a mail server built for integration testing. Getting Greenmail to work with SSL/TLS in an integration testing environment is a bit tricky; I have [instructions on my blog](https://crispinstichart.github.io/using-SSL-in-greenmail-docker-container/).

We have code coverage provided by the LLVM source-based coverage tooling in Rust's nightly branch.

TODO: Make a custom Greenmail docker image that contains a custom certificate, and get that integrated with our GitHub Actions pipeline, using it as a [service container](https://docs.github.com/en/actions/using-containerized-services/about-service-containers).  