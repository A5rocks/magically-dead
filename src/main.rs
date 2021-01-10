#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use ring::signature;
use std::convert::{Infallible, TryFrom, TryInto};
use std::net::SocketAddr;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to do ctrl+c handling")
}

const DISCORD_PUBLIC_KEY_STRING: &str =
    "144a270b8f0562d7dc39a8f23e711620b2ba4aff5decc92fcbdcc18955c7f3ea";

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, magic::MagicError> {
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

            let body_string: &str = std::str::from_utf8(&full_body)?;

            let p: magic::request_types::RawInteraction = serde_json::from_str(body_string)?;

            if p.interaction_type == 1 {
                *resp.body_mut() = serde_json::json!({
                    "type": 1
                })
                .to_string()
                .into();
                return Ok(resp);
            } else {
                let interaction = magic::request_types::Interaction::try_from(p)?;

                return Ok(Response::new(
                    magic::handle_interaction(interaction).await?.try_into()?,
                ));
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
        Ok(response) => Ok(response),
        Err(err) => {
            let mut response = Response::default();
            *response.status_mut() = StatusCode::BAD_REQUEST;
            *response.body_mut() = format!("{}", err).into();

            Ok(response)
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
