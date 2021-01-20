pub mod request_types;
pub mod response_types;

use response_types::{Data, InteractionResponse};
use serde::{Deserialize, Serialize};
use sled::{transaction::ConflictableTransactionResult, Transactional};
use std::convert::Infallible;
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
    lobbies: sled::Tree,
    players: sled::Tree,
}

fn encode_lobby(lobby: &Lobby) -> Vec<u8> {
    bincode::serialize(lobby).expect("could not serialize lobby?")
}

fn decode_lobby(lobby: &[u8]) -> Lobby {
    bincode::deserialize(lobby).expect("bad lobby state.")
}

impl Database {
    pub fn make(db: sled::Db) -> Self {
        Self {
            lobbies: db
                .open_tree("lobbies")
                .expect("was not able to open lobby tree"),
            players: db
                .open_tree("players")
                .expect("was not able to open player tree"),
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

impl From<sled::Error> for MagicError {
    fn from(s: sled::Error) -> Self {
        eprintln!("sled error says: {:?}", s);
        Self::SledError
    }
}

pub async fn create_lobby(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let player_id = interaction.clone().member().user().id();
    let result = (&db.lobbies, &db.players)
        .transaction(|(lobbies, players)| {
            let ok = ConflictableTransactionResult::<sled::Result<&'static str>, Infallible>::Ok;
            let player = players.get(&player_id)?;
            let lobby_id_val = interaction.clone().channel_id();
            let lobby_id = lobby_id_val.as_str();
            let cur_lobby = lobbies
                .get(lobby_id)?
                .map(|thing| decode_lobby(thing.as_ref()));

            // if there's a lobby already...
            // TODO: make this do some extra work for UX (leaving, etc.)
            if let Some(_lobby) = cur_lobby {
                // we do some extra work for error messages.
                return if let Some(id) = player {
                    if id == lobby_id {
                        ok(Ok(
                            "you're already in that lobby! (`hijack` will be implemented soon:tm:)",
                        ))
                    } else {
                        ok(Ok("you're in another lobby!"))
                    }
                } else {
                    ok(Ok("a lobby already exists in this channel! try /join!"))
                };
            };

            if let Some(_player_lobby_id) = player {
                // the player is in a lobby
                // so... are they the owner of their old lobby?
                // TODO: delete / leave old lobby
                return ok(Ok("tell a5 to do this."));
            }

            lobbies.insert(
                lobby_id,
                encode_lobby(&Lobby {
                    creator: player_id.clone(),
                    players: vec![player_id.clone()],
                }),
            )?;
            players.insert(player_id.as_str(), lobby_id)?;

            ConflictableTransactionResult::<sled::Result<&'static str>, Infallible>::Ok(Ok(
                "all systems are a go.",
            ))
        })
        .expect("tx error")?;

    Ok(InteractionResponse::create(
        3,
        Data::content(format!("create lobby: {}", result)),
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
        "801198519263559690" => Ok(InteractionResponse::create(
            3,
            Data::content("leave lobby".to_string()),
        )),
        _ => Ok(InteractionResponse::create(
            4,
            Data::content("Command not set up.".to_string()),
        )),
    }
}
