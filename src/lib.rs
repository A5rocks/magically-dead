use std::{convert::TryFrom, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawInteraction {
    id: String,
    // todo: better type for this...?
    #[serde(rename = "type")]
    pub interaction_type: u8,
    data: Option<ApplicationCommandData>,
    guild_id: Option<String>,
    channel_id: Option<String>,
    member: Option<GuildMember>,
    token: String,
    version: u8,
}

impl TryFrom<RawInteraction> for Interaction {
    type Error = MagicError;

    fn try_from(value: RawInteraction) -> Result<Self, Self::Error> {
        if value.interaction_type == 1 {
            Err(MagicError::GenericError)
        } else {
            Ok(Self {
                id: value.id,
                interaction_type: value.interaction_type,
                data: value.data,
                guild_id: value.guild_id.ok_or(MagicError::GenericError)?,
                channel_id: value.channel_id.ok_or(MagicError::GenericError)?,
                member: value.member.ok_or(MagicError::GenericError)?,
                token: value.token,
                version: value.version,
            })
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Interaction {
    id: String,
    // todo: better type for this...?
    #[serde(rename = "type")]
    interaction_type: u8,
    data: Option<ApplicationCommandData>,
    guild_id: String,
    channel_id: String,
    member: GuildMember,
    token: String,
    version: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApplicationCommandData {
    id: String,
    name: String,
    options: Option<Vec<ApplicationCommandDataOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ApplicationCommandDataValue {
    String(String),
    // it can be higher, but oh well.
    Number(i128),
    Boolean(bool),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ApplicationCommandDataOption {
    Value {
        name: String,
        value: ApplicationCommandDataValue,
    },
    Nested {
        name: String,
        options: Vec<ApplicationCommandDataOption>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct GuildMember {
    user: User,
    nick: Option<String>,
    roles: Vec<String>,
    // this is unneeded, let's not include it
    // joined_at:
    // neither is this
    // premium_since:
    deaf: bool,
    mute: bool,
    pending: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    username: String,
    discriminator: String,
    bot: Option<bool>,
    avatar: Option<String>,
    system: Option<bool>,
    // todo: maybe I should do this?
    public_flags: u64,
}

#[derive(Debug)]
pub enum MagicError {
    WeirdHTTPError(String),
    StringConversion,
    JSONParsing(String),
    // error for things idk about yet
    GenericError,
}

impl Error for MagicError {}

impl fmt::Display for MagicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WeirdHTTPError(location) => {
                write!(f, "Some weird hyper error happened while {}.", location)
            }
            Self::StringConversion => write!(
                f,
                "An error occurred while converting your body to a string."
            ),
            Self::JSONParsing(err) => write!(f, "{}", err),
            Self::GenericError => write!(f, "An error occurred!"),
        }
    }
}

impl From<hyper::Error> for MagicError {
    fn from(s: hyper::Error) -> Self {
        println!("Hyper error says: {:?}", s);
        Self::WeirdHTTPError("buffering body".to_string())
    }
}

impl From<std::str::Utf8Error> for MagicError {
    fn from(s: std::str::Utf8Error) -> Self {
        println!("String error says: {:?}", s);
        Self::StringConversion
    }
}

impl From<serde_json::Error> for MagicError {
    fn from(s: serde_json::Error) -> Self {
        Self::JSONParsing(format!("JSON error: {}", s))
    }
}

pub async fn handle_interaction(interaction: Interaction) {
    println!("{:?}", interaction);
}
