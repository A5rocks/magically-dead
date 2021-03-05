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

fn create_lobby(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let mut options: Vec<request_types::ApplicationCommandDataOption> = interaction
        .clone()
        .data()
        .expect("create lobby missing `data`")
        .options()
        .unwrap_or_default();

    options.retain(|option| {
        match option {
            request_types::ApplicationCommandDataOption::Value { name, value } => {
                if let request_types::ApplicationCommandDataValue::Boolean(_) = value {
                    name == "hijack"
                } else {
                    false
                }
            }
            request_types::ApplicationCommandDataOption::Nested { .. } => false, // TODO: maybe recurse? (flatmap)
        }
    });

    let options: Vec<_> = options
        .iter()
        .map(|option| {
            if let request_types::ApplicationCommandDataOption::Value { value, .. } = option {
                if let request_types::ApplicationCommandDataValue::Boolean(bool) = value {
                    // get rid of that nasty reference (turns into a double reference with `Vec.get`)
                    *bool
                } else {
                    panic!("impossible state.")
                }
            } else {
                panic!("impossible state.")
            }
        })
        .collect();

    let hijacking = options
        .get(0)
        .map(|thing| thing.to_owned())
        .unwrap_or(false);

    // hijacking requires MANAGE_MESSAGES
    if hijacking
        && interaction
            .clone()
            .member()
            .permissions()
            .parse::<u128>()
            .expect("bad permissions int")
            >> 13
            & 1
            == 0
    {
        return Ok(InteractionResponse::create(
            3,
            Data::content(format!(
                "create lobby: {}",
                "you don't have permissions required to hijack"
            )),
        ));
    };

    let player_id = interaction.clone().member().user().id();
    let result = (&db.lobbies, &db.players)
        .transaction(|(lobbies, players)| {
            let player = players.get(&player_id)?;
            let lobby_id_val = interaction.clone().channel_id();
            let lobby_id = lobby_id_val.as_str();
            let cur_lobby = lobbies
                .get(lobby_id)?
                .map(|thing| decode_lobby(thing.as_ref()));

            // if there's a lobby already...
            // TODO: make this do some extra work for UX (leaving, etc.)
            if let Some(id) = player {
                // we do some extra work for error messages.
                return if id == lobby_id {
                    Ok(Ok("you're already in that lobby!"))
                } else {
                    Ok(Ok("you're in another lobby!"))
                };
            };

            let hijacking = if cur_lobby.is_none() {
                // it doesn't matter, let's simplify logic
                false
            } else {
                hijacking
            };

            if cur_lobby.is_some() && !hijacking {
                return Ok(Ok("a lobby already exists in this channel! try /join!"));
            }

            if player.is_some() {
                // the player is in a lobby
                // so... are they the owner of their old lobby?
                // TODO: delete / leave old lobby
                return Ok(Ok("tell a5 to do this."));
            }

            if !hijacking {
                lobbies.insert(
                    lobby_id,
                    encode_lobby(&Lobby {
                        creator: player_id.clone(),
                        players: vec![player_id.clone()],
                    }),
                )?;
            } else {
                let mut prev_players = cur_lobby.map(|thing| thing.players).unwrap_or_default();
                prev_players.push(player_id.clone());

                lobbies.insert(
                    lobby_id,
                    encode_lobby(&Lobby {
                        creator: player_id.clone(),
                        players: prev_players,
                    }),
                )?;
            }
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

fn join_lobby(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let player_id_val = interaction.clone().member().user().id();
    let player_id = player_id_val.as_str();
    let lobby_id_val = interaction.channel_id();
    let lobby_id = lobby_id_val.as_str();

    let result = (&db.lobbies, &db.players)
        .transaction(|(lobbies, players)| {
            let player = players.get(player_id)?;

            // in a lobby already?
            if player.is_some() {
                // TODO: leaving for UX (requires some abstraction)
                return Ok(Ok("leave your previous lobby first"));
            }

            let lobby_str = lobbies.get(lobby_id)?;

            // is there not a lobby?
            if lobby_str.is_none() {
                // TODO: making a lobby for UX (requires some abstraction)
                return Ok(Ok("this channel does not have a lobby, make one instead?"));
            };

            let mut lobby = decode_lobby(&lobby_str.unwrap());

            lobby.players.push(player_id_val.clone());

            lobbies.insert(
                lobby_id,
                encode_lobby(&Lobby {
                    creator: lobby.creator,
                    players: lobby.players,
                }),
            )?;

            players.insert(player_id, lobby_id)?;

            ConflictableTransactionResult::<sled::Result<&'static str>, Infallible>::Ok(Ok(
                "joined the lobby.",
            ))
        })
        .expect("tx error")?;

    Ok(InteractionResponse::create(
        3,
        Data::content(format!("join lobby: {}", result)),
    ))
}

fn kill_player(
    _interaction: request_types::Interaction,
    _db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    Ok(InteractionResponse::create(
        3,
        Data::content("kill player".to_string()),
    ))
}

fn vote_player(
    _interaction: request_types::Interaction,
    _db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    Ok(InteractionResponse::create(
        3,
        Data::content("vote player".to_string()),
    ))
}

fn leave_lobby(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let player_id = interaction.clone().member().user().id();
    let lobby_id_val = interaction.channel_id();
    let lobby_id = lobby_id_val.as_str();

    let result = (&db.players, &db.lobbies)
        .transaction(|(players, lobbies)| {
            let player = players.get(&player_id)?;

            if player.is_none()
                || std::str::from_utf8(player.unwrap().as_ref()).unwrap() != lobby_id
            {
                return Ok(Ok("you need to be in a lobby to leave it."));
            };

            players.remove(player_id.as_str())?;

            let lobby_str = lobbies.get(lobby_id)?;

            if lobby_str.is_none() {
                return Ok(Ok("there was no such lobby???"));
            };

            let mut lobby = decode_lobby(&lobby_str.unwrap());

            if lobby.creator.as_str() == player_id {
                // you're the creator
                for player in lobby.players {
                    if player != player_id {
                        players.remove(player.as_str())?;
                    };
                }

                lobbies.remove(lobby_id)?;

                Ok(Ok("disbanded the lobby!"))
            } else {
                // you're not the creator
                lobby.players.retain(|n| n == &player_id);
                lobbies.insert(
                    lobby_id,
                    encode_lobby(&Lobby {
                        creator: lobby.creator,
                        players: lobby.players,
                    }),
                )?;

                ConflictableTransactionResult::<sled::Result<&'static str>, Infallible>::Ok(Ok(
                    "left the lobby!",
                ))
            }
        })
        .expect("tx error")?;

    Ok(InteractionResponse::create(
        3,
        Data::content(format!("leave lobby: {}", result)),
    ))
}

pub async fn handle_interaction(
    interaction: request_types::Interaction,
    db: Database,
) -> Result<response_types::InteractionResponse, MagicError> {
    let data = interaction.clone().data().ok_or(MagicError::GenericError)?;

    match data.id().as_str() {
        "796995810038382642" => create_lobby(interaction, db),
        "796996870815744010" => join_lobby(interaction, db),
        "796999207046742027" => kill_player(interaction, db),
        "796999927782834176" => vote_player(interaction, db),
        "801198519263559690" => leave_lobby(interaction, db),
        _ => Ok(InteractionResponse::create(
            4,
            Data::content("Command not set up.".to_string()),
        )),
    }
}
