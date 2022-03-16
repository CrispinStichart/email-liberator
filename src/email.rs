use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Email {
    pub sender: String,
    pub subject: String,
    pub body: String,
}

impl Email {
    pub fn from_fetch(msg: &imap::types::Fetch) -> Email {
        Email {
            // jesus christ :|
            // This looks gnarly as heck; there's got to be a better way.
            // Is there a macro that can unwrap multiple nested items?
            sender: msg
                .envelope()
                .unwrap()
                .sender
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| format!("{:?}", a))
                .collect::<Vec<String>>()
                .join(" "),
            subject: msg
                .envelope()
                .unwrap()
                .subject
                .as_ref()
                .and_then(|cow| std::str::from_utf8(cow).ok())
                .unwrap()
                .to_string(),
            body: std::str::from_utf8(msg.body().unwrap())
                .unwrap()
                .to_string(),
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Email> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
