use std::convert::TryFrom;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
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
    type Error = crate::MagicError;

    fn try_from(value: RawInteraction) -> Result<Self, Self::Error> {
        if value.interaction_type == 1 {
            Err(crate::MagicError::GenericError)
        } else {
            Ok(Self {
                id: value.id,
                interaction_type: value.interaction_type,
                data: value.data,
                guild_id: value.guild_id.ok_or(crate::MagicError::GenericError)?,
                channel_id: value.channel_id.ok_or(crate::MagicError::GenericError)?,
                member: value.member.ok_or(crate::MagicError::GenericError)?,
                token: value.token,
                version: value.version,
            })
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
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

impl Interaction {
    // i cba to make this a reference like it should be...
    // todo: ^
    pub fn data(self) -> Option<ApplicationCommandData> {
        self.data
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn guild_id(self) -> String {
        self.guild_id
    }

    pub fn channel_id(self) -> String {
        self.channel_id
    }

    pub fn member(self) -> GuildMember {
        self.member
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApplicationCommandData {
    id: String,
    name: String,
    options: Option<Vec<ApplicationCommandDataOption>>,
}

impl ApplicationCommandData {
    pub fn id(self) -> String {
        self.id
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum ApplicationCommandDataValue {
    String(String),
    // it can be higher, but oh well.
    Number(i128),
    Boolean(bool),
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct GuildMember {
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

impl GuildMember {
    pub fn user(self) -> User {
        self.user
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    id: String,
    username: String,
    discriminator: String,
    bot: Option<bool>,
    avatar: Option<String>,
    system: Option<bool>,
    // todo: maybe I should do this?
    public_flags: u64,
}

impl User {
    pub fn id(self) -> String {
        self.id
    }
}
