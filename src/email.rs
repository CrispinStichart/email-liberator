use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Email {
    pub sender: Vec<Address>,
    pub subject: String,
    pub body: String,
}

#[derive(Serialize, Deserialize)]
pub struct Address {
    pub name: Option<String>,
    pub adl: Option<String>,
    pub mailbox: Option<String>,
    pub host: Option<String>,
}

impl Address {
    fn from_imap_address(address: &imap_proto::Address) -> Result<Address> {
        Ok(Address {
            name: address
                .name
                .as_ref()
                .and_then(|a| Some(String::from_utf8(a.to_vec()).unwrap())),
            adl: address
                .adl
                .as_ref()
                .and_then(|a| Some(String::from_utf8(a.to_vec()).unwrap())),
            mailbox: address
                .mailbox
                .as_ref()
                .and_then(|a| Some(String::from_utf8(a.to_vec()).unwrap())),
            host: address
                .mailbox
                .as_ref()
                .and_then(|a| Some(String::from_utf8(a.to_vec()).unwrap())),
        })
    }
}

impl Email {
    pub fn from_fetch(msg: &imap::types::Fetch) -> Result<Email> {
        let envelope = msg
            .envelope()
            .ok_or(anyhow!("No envlope in fetch"))?;

        Ok(Email {
            // jesus christ :|
            // This looks gnarly as heck; there's got to be a better way.
            // Is there a macro that can unwrap multiple nested items?
            sender: envelope
                .from
                .as_ref()
                .unwrap()
                .as_slice()
                .iter()
                .map(|a| Address::from_imap_address(a).expect("Couldn't create address struct"))
                .collect(),
            subject: envelope
                .subject
                .as_ref()
                .and_then(|cow| std::str::from_utf8(cow).ok())
                .unwrap()
                .to_string(),
            body: std::str::from_utf8(msg.body().unwrap())
                .unwrap()
                .to_string(),
        })
    }

    pub fn from_json(json: &str) -> serde_json::Result<Email> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
