use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub uid: u32,
    pub actions: Vec<Action>,
    pub stop: Option<bool>,
}

// TODO: check actions vector for equality
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid && self.stop == other.stop
    }
}

impl Eq for Message {}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum Action {
    Move(String),
    Delete,
    Label,
}

impl Message {
    pub fn from_json(json: &str) -> serde_json::Result<Message> {
        serde_json::from_str(json)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_and_forth() -> Result<()> {
        let msg = Message {
            uid: 69,
            actions: vec![Action::Delete],
            stop: None,
        };

        assert_eq!(msg, Message::from_json(&msg.to_string())?);

        Ok(())
    }
}
