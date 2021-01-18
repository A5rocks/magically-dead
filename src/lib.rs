pub mod request_types;
pub mod response_types;

use response_types::{Data, InteractionResponse};
use serde::{Deserialize, Serialize};
use sled_extensions::bincode::Tree;
use sled_extensions::DbExt;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum MagicError {
    WeirdHTTPError(String),
    StringConversion,
    JSONParsing(String),
    // error for things idk about yet
    GenericError,
    SledError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lobby {
    creator: String,
    players: Vec<String>,
}

#[derive(Clone)]
pub struct Database {
    // TODO: this
    lobbies: std::sync::Arc<Tree<Lobby>>,
}

impl Database {
    pub fn make(db: sled_extensions::Db) -> Self {
        Self {
            lobbies: std::sync::Arc::from(
                db.open_bincode_tree("lobbies")
                    .expect("was not able to open lobby tree"),
            ),
        }
    }
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
            Self::SledError => write!(f, "A filesystem error happened with sled!"),
        }
    }
}

impl From<hyper::Error> for MagicError {
    fn from(s: hyper::Error) -> Self {
        eprintln!("Hyper error says: {:?}", s);
        Self::WeirdHTTPError("buffering body".to_string())
    }
}

impl From<std::str::Utf8Error> for MagicError {
    fn from(s: std::str::Utf8Error) -> Self {
        eprintln!("String error says: {:?}", s);
        Self::StringConversion
    }
}

impl From<serde_json::Error> for MagicError {
    fn from(s: serde_json::Error) -> Self {
        Self::JSONParsing(format!("JSON error: {}", s))
    }
}

impl From<sled_extensions::Error> for MagicError {
    fn from(s: sled_extensions::Error) -> Self {
        eprintln!("sled error says: {:?}", s);
        Self::SledError
    }
}

pub async fn create_lobby(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    db.lobbies
        .as_ref()
        .transaction(|db| {
            let prior_lobby = db.get(interaction.clone().channel_id())?;
            println!("Prior lobby: {:?}", prior_lobby);

            db.insert(
                interaction.clone().channel_id().as_str(),
                Lobby {
                    creator: interaction.clone().member().user().id(),
                    players: vec![interaction.clone().member().user().id()],
                },
            )?
            .expect("todo!");

            Ok(Ok(()))
        })
        .expect("tx error")?;

    Ok(InteractionResponse::create(
        3,
        Data::content("create lobby".to_string()),
    ))
}

pub async fn handle_interaction(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let data = interaction.clone().data().ok_or(MagicError::GenericError)?;

    match data.id().as_str() {
        "796995810038382642" => Ok(create_lobby(interaction, db).await?),
        "796996870815744010" => Ok(InteractionResponse::create(
            3,
            Data::content("join lobby".to_string()),
        )),
        "796999207046742027" => Ok(InteractionResponse::create(
            3,
            Data::content("kill player".to_string()),
        )),
        "796999927782834176" => Ok(InteractionResponse::create(
            3,
            Data::content("vote player".to_string()),
        )),
        _ => Ok(InteractionResponse::create(
            4,
            Data::content("Command not set up.".to_string()),
        )),
    }
}
