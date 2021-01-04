use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use ring::signature;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::{error::Error, fmt, str};

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to do ctrl+c handling")
}

#[derive(Serialize, Deserialize, Debug)]
struct RawInteraction {
    id: String,
    // todo: better type for this...?
    #[serde(rename = "type")]
    interaction_type: u8,
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
            if let None = value.guild_id {
                return Err(MagicError::GenericError);
            }

            if let None = value.channel_id {
                return Err(MagicError::GenericError);
            }

            if let None = value.member {
                return Err(MagicError::GenericError);
            }

            Ok(Interaction {
                id: value.id,
                interaction_type: value.interaction_type,
                data: value.data,
                guild_id: value
                    .guild_id
                    .ok_or_else(|| return MagicError::GenericError)?,
                channel_id: value
                    .channel_id
                    .ok_or_else(|| return MagicError::GenericError)?,
                member: value
                    .member
                    .ok_or_else(|| return MagicError::GenericError)?,
                token: value.token,
                version: value.version,
            })
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Interaction {
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
enum MagicError {
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
            MagicError::WeirdHTTPError(location) => {
                write!(f, "Some weird hyper error happened while {}.", location)
            }
            MagicError::StringConversion => write!(
                f,
                "An error occurred while converting your body to a string."
            ),
            MagicError::JSONParsing(err) => write!(f, "{}", err),
            MagicError::GenericError => write!(f, "An error occurred!"),
        }
    }
}

impl From<hyper::Error> for MagicError {
    fn from(s: hyper::Error) -> MagicError {
        println!("Hyper error says: {:?}", s);
        MagicError::WeirdHTTPError("buffering body".to_string())
    }
}

impl From<str::Utf8Error> for MagicError {
    fn from(s: str::Utf8Error) -> MagicError {
        println!("String error says: {:?}", s);
        MagicError::StringConversion
    }
}

impl From<serde_json::Error> for MagicError {
    fn from(s: serde_json::Error) -> MagicError {
        MagicError::JSONParsing(format!("JSON error: {}", s))
    }
}

const DISCORD_PUBLIC_KEY_STRING: &str =
    "144a270b8f0562d7dc39a8f23e711620b2ba4aff5decc92fcbdcc18955c7f3ea";

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, MagicError> {
    let public_key = signature::UnparsedPublicKey::new(
        &signature::ED25519,
        hex::decode(DISCORD_PUBLIC_KEY_STRING).expect("invalid hex for public key"),
    );
    let mut resp = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *resp.body_mut() = "Hello, world!".into();
        }
        (&Method::POST, "/") => {
            let timestamp = req.headers().get("x-signature-timestamp");

            if timestamp == None {
                *resp.body_mut() = "No timestamp!".into();
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }

            let timestamp_string = timestamp.unwrap().to_str();

            if let Err(_e) = timestamp_string {
                *resp.body_mut() = "Invalid timestamp.".into();
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }

            let signature = req.headers().get("x-signature-ed25519");

            if signature == None {
                *resp.body_mut() = "No signature!".into();
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }

            let signature_string = signature.unwrap().to_str();

            if let Err(_e) = signature_string {
                *resp.body_mut() = "Invalid signature.".into();
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }

            let signature_bytes = hex::decode(signature_string.unwrap());

            if let Err(_e) = signature_bytes {
                *resp.body_mut() = "Invalid hex for signature.".into();
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }

            let mut verified_body: Vec<u8> = timestamp_string.unwrap().into();
            let body = hyper::body::to_bytes(req.into_body()).await?;
            verified_body.append(&mut body.to_vec());

            let verified = public_key.verify(&verified_body, &signature_bytes.unwrap());

            if let Err(_e) = verified {
                *resp.body_mut() = "Bad signature.".into();
                *resp.status_mut() = StatusCode::UNAUTHORIZED;
                return Ok(resp);
            }

            let full_body = body.to_vec();

            let body_string: &str = str::from_utf8(&full_body)?;

            let p: RawInteraction = serde_json::from_str(body_string)?;

            if p.interaction_type == 1 {
                // todo: not so dumb version
                *resp.body_mut() = "{\"type\":1}".into();
                return Ok(resp);
            } else {
                let interaction = Interaction::try_from(p)?;

                println!("{:?}", interaction);
            }
        }
        _ => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Ok(resp)
}

async fn error_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match handle_request(req).await {
        Ok(res) => Ok(res),
        Err(err) => {
            let mut resp = Response::default();
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            *resp.body_mut() = format!("{}", err).into();

            Ok(resp)
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(error_handler)) });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
