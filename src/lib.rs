pub mod request_types;
pub mod response_types;

use std::{error::Error, fmt};

use response_types::{Data, InteractionResponse};

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

pub async fn handle_interaction(
    interaction: request_types::Interaction,
) -> Result<response_types::InteractionResponse, MagicError> {
    let data = interaction.data().ok_or(MagicError::GenericError)?;

    match &data.id()[..] {
        "796995810038382642" => Ok(InteractionResponse::create(
            3,
            Data::content("create lobby".to_string()),
        )),
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
