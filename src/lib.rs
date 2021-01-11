pub mod request_types;
pub mod response_types;

use sqlx::PgPool;
use std::{error::Error, fmt};
use tokio_compat_02::FutureExt;

use response_types::{Data, InteractionResponse};

#[derive(Debug)]
pub enum MagicError {
    WeirdHTTPError(String),
    StringConversion,
    JSONParsing(String),
    SQLError,
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
            Self::SQLError => write!(f, "An error occurred in SQL!"),
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

impl From<sqlx::Error> for MagicError {
    fn from(s: sqlx::Error) -> Self {
        eprintln!("SQL error says: {:?}", s);
        Self::SQLError
    }
}

pub async fn handle_interaction(
    interaction: request_types::Interaction,
    pool: PgPool,
) -> Result<response_types::InteractionResponse, MagicError> {
    sqlx::query("INSERT INTO DOES_IT_WORK VALUES ($1)")
        .bind(interaction.id().parse::<i64>().unwrap_or(5))
        .execute(&pool)
        .compat()
        .await
        .expect("whoops");

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
