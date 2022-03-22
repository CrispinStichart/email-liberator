use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Email {
    pub sender: Vec<Address>,
    pub subject: String,
    pub body: String,
    pub uid: u32,
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
            uid: msg.uid.unwrap(),
        })
    }

    pub fn from_json(json: &str) -> serde_json::Result<Email> {
        serde_json::from_str(json.trim())
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parsing() -> Result<()> {
        let input = r#"{"uid": 69, "sender":[{"name":null,"adl":null,"mailbox":"sender","host":"sender"}],"subject":"My first e-mail","body":"Return-Path: <sender@localhost>\r\nReceived: from 172.17.0.1 (HELO DESKTOP-D0BUERJ); Thu Mar 17 18:48:37 UTC 2022\r\nSubject: My first e-mail\r\nTo: <test@greenmail.com>\r\nFrom: <sender@localhost>\r\nDate: Thu, 17 Mar 2022 13:48:37 -0500\r\nMIME-Version: 1.0\r\nMessage-ID: <eb182115-fac3-4bda-9381-3f04084bc8cc.lettre@localhost>\r\nContent-Type: multipart/mixed; boundary=z9Y94WiriL0hfCPAzgC2ohO6XkpuUi\r\n\r\n\r\n--z9Y94WiriL0hfCPAzgC2ohO6XkpuUi\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nHello world from SMTP\r\n\r\n--z9Y94WiriL0hfCPAzgC2ohO6XkpuUi--\r\n\r\n"}"#;
        let email = Email::from_json(input)?;
        assert_eq!(email.subject, "My first e-mail");

        Ok(())
    }
}
