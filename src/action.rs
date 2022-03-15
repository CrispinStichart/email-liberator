use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub uid: u32,
    pub actions: Vec<Action>,
    pub stop: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
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
