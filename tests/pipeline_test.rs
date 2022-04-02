use anyhow::Result;
pub mod utils;
use utils::*;

// test emails needed:
// an email that will get caught by the spam filter
// an email that won't but has some matching text before the cutoff point
// an email that is totally unrelated and won't match the filter

// need helper function to take multiple test emails and sum the wordcount
// of the their bodies

#[test]
fn run_pipeline() -> Result<()> {
    // delete the wordcount file, /tmp/email_word_count
    // login using example_scripts/example_config.toml

    // spin up three process
    // connect them with pipes
    // send a couple test emails

    // check that the email that was supposed to be deleted actually was
    // check the wordcount file matches the count we made

    Ok(())
}
