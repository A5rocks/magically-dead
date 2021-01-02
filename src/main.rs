use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};
use futures::TryStreamExt as _;

async fn shutdown_signal() {
	tokio::signal::ctrl_c()
		.await
		.expect("failed to do ctrl+c handling")
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	let mut resp = Response::new(Body::empty());

	match (req.method(), req.uri().path()) {
		(&Method::GET, "/") => {
			*resp.body_mut() = "Hello, world!".into();
		},
		(&Method::POST, "/") => {
			*resp.body_mut() = req.into_body();
		},
		(&Method::POST, "/uppercase") => {
			let stream = req
				.into_body()
				.map_ok(|chunk| {
					chunk.iter()
						.map(|byte| byte.to_ascii_uppercase())
						.collect::<Vec<u8>>()
				});

			*resp.body_mut() = Body::wrap_stream(stream);
		},
		(&Method::POST, "/reverse") => {
			let full_body = hyper::body::to_bytes(req.into_body()).await?;

			let reversed = full_body.iter()
				.rev()
				.cloned()
				.collect::<Vec<u8>>();

			*resp.body_mut() = reversed.into();
		}
		_ => {
			*resp.status_mut() = StatusCode::NOT_FOUND;
		}
	}

	Ok(resp)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    let make_svc = make_service_fn(|_conn| async {
    	Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr)
    	.serve(make_svc)
    	.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
    	eprintln!("server error: {}", e);
    }
}
