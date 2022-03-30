use anyhow::{anyhow, Context, Result};
use mailparse;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Email {
    /// Sender is a Vec because rfc6854 allows multiple senders, we use an
    /// option because even no senders at all is allowed.
    pub sender: Vec<Option<String>>,
    pub subject: String,
    pub body: String,
    pub uid: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Address {
    /// Human readable name, e.g. "Jane Smith"
    pub name: Option<String>,
    /// Route, I think?
    pub adl: Option<String>,
    /// The part before the @ sign in an address
    pub mailbox: Option<String>,
    /// The domain of the address
    pub host: Option<String>,
}

impl Address {
    fn from_imap_address(address: &imap_proto::Address) -> Result<Address> {
        // TODO: can I simplify this somehow?
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
                .host
                .as_ref()
                .and_then(|a| Some(String::from_utf8(a.to_vec()).unwrap())),
        })
    }

    /// Returns the address as a "normal" email, e.g. bob@gmail.com. The sender
    /// can theoretically be empty; rfc6854 gives the example of an automated
    /// system that doesn't support replies. In the real world I don't know how
    /// much use that sees -- in my experience, people just use a
    /// no-reply@whatever.com style address.
    fn to_simple(&self) -> Option<String> {
        if self
            .mailbox
            .is_some()
            && self.host.is_some()
        {
            Some(format!(
                "{}@{}",
                &self
                    .mailbox
                    .as_ref()
                    .unwrap(),
                &self
                    .host
                    .as_ref()
                    .unwrap()
            ))
        } else {
            None
        }
    }
}

impl Email {
    pub fn from_fetch(msg: &imap::types::Fetch) -> Result<Email> {
        let envelope = msg
            .envelope()
            .ok_or(anyhow!("No envlope in fetch"))?;

        let body = msg.body().unwrap();
        let parsed = mailparse::parse_mail(body)?;
        let body = parsed
            .subparts
            .get(0)
            .context("uh-oh, looks like it wasn't a multi-part message!")?
            .get_body()
            .context(format!("No body in message? WTF? UID={:?}", msg.uid))?;

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
                .map(|a| {
                    Address::from_imap_address(a)
                        .expect("Couldn't create address struct")
                        .to_simple()
                })
                .collect(),
            subject: envelope
                .subject
                .as_ref()
                .and_then(|cow| std::str::from_utf8(cow).ok())
                .unwrap()
                .to_string(),
            body,
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
    fn from_json() -> Result<()> {
        let input = concat!(
            r#"{"sender":["sender.bob@gmail.com"],"#,
            r#""subject":"My first e-mail","#,
            r#""body":"Hello world from SMTP\r\n\r\n","uid":16}"#
        );
        let email = Email::from_json(input)?;
        assert_eq!(email.subject, "My first e-mail");
        assert_eq!(
            email.sender[0]
                .as_ref()
                .unwrap(),
            "sender.bob@gmail.com"
        );
        assert_eq!(email.body, "Hello world from SMTP\r\n\r\n");
        assert_eq!(email.uid, 16);

        Ok(())
    }

    #[test]
    fn to_json() -> Result<()> {
        let expected_json = concat!(
            r#"{"sender":["sender.bob@gmail.com"],"#,
            r#""subject":"My first e-mail","#,
            r#""body":"Hello world from SMTP\r\n\r\n","uid":16}"#
        );
        let email = Email {
            sender: vec![Some("sender.bob@gmail.com".to_string())],
            subject: "My first e-mail".to_string(),
            body: "Hello world from SMTP\r\n\r\n".to_string(),
            uid: 16,
        };

        assert_eq!(expected_json, email.to_json()?);

        Ok(())
    }
}
