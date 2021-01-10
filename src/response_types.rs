use std::convert::TryInto;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct InteractionResponse {
    // todo: is there a better way to do this
    #[serde(rename = "type")]
    response_type: u8,
    data: Option<Data>,
}

impl InteractionResponse {
    pub fn create_optional(r#type: u8, data: Option<Data>) -> Self {
        Self {
            response_type: r#type,
            data,
        }
    }

    pub fn create(r#type: u8, data: Data) -> Self {
        Self {
            response_type: r#type,
            data: Some(data),
        }
    }
}

impl TryInto<hyper::body::Body> for InteractionResponse {
    type Error = crate::MagicError;

    fn try_into(self) -> Result<hyper::body::Body, Self::Error> {
        Ok(serde_json::to_string(&self)?.into())
    }
}

#[derive(Serialize, Debug)]
pub enum ResponseType {
    Pong = 1,
    Acknowledge = 2,
    ChannelMessage = 3,
    ChannelMessageWithSource = 4,
    AcknowledgeWithSource = 5,
}

#[derive(Serialize, Debug)]
pub struct Data {
    tts: Option<bool>,
    content: String,
    // we don't actually need this, we won't be using embeds
    // embeds: Option<Vec<Embed>>,
    allowed_mentions: AllowedMentions,
    // the only one that works is "64"
    flags: Option<u16>,
}

impl Data {
    pub fn content(content: String) -> Self {
        Self {
            content,
            tts: None,
            allowed_mentions: AllowedMentions::default(),
            flags: None,
        }
    }

    pub fn ephemeral_content(content: String) -> Self {
        Self {
            content,
            tts: None,
            allowed_mentions: AllowedMentions::default(),
            flags: Some(64),
        }
    }

    pub fn pinging_content(content: String) -> Self {
        Self {
            content,
            tts: None,
            allowed_mentions: AllowedMentions::all(),
            flags: None,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AllowedMentions {
    parse: Vec<String>,
    roles: Option<Vec<String>>,
    users: Option<Vec<String>>,
    replied_user: Option<bool>,
}

impl AllowedMentions {
    pub fn none() -> Self {
        Self {
            parse: vec![],
            roles: None,
            users: None,
            replied_user: None,
        }
    }

    pub fn all() -> Self {
        // even then we don't want @everyone !
        Self {
            parse: vec!["roles".to_string(), "users".to_string()],
            roles: None,
            users: None,
            replied_user: None,
        }
    }

    pub fn everyone() -> Self {
        Self {
            parse: vec!["everyone".to_string()],
            roles: None,
            users: None,
            replied_user: None,
        }
    }
}

impl Default for AllowedMentions {
    fn default() -> Self {
        // you can choose!
        // all()
        Self::none()
    }
}
