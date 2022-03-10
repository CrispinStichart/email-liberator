# Automatic Mail Filter

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