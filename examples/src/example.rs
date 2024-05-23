#![deny(warnings)]

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::Frame, header::HeaderValue, server::conn::http2, service::service_fn, Method, Request, Response,
    StatusCode,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let executor = TokioExecutor::new();
        tokio::task::spawn(async move {
            if let Err(err) = http2::Builder::new(executor)
                .serve_connection(io, service_fn(service))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn service(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::POST, "/discover") => {
            let manifest = r#"{
  "protocolMode": "BIDI_STREAM",
  "minProtocolVersion": 1,
  "maxProtocolVersion": 1,
  "services": [
    {
      "name": "Greeter",
      "ty": "SERVICE",
      "handlers": [
        {
          "name": "greet",
          "ty": "EXCLUSIVE"
        }
      ]
    }
  ]
}"#;
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }
            //let content_type = content_type.to_str().unwrap();
            let mut response = Response::new(full(manifest));
            //response.headers_mut().insert(":status", HeaderValue::from(200));
            response
                .headers_mut()
                .insert("content-type", HeaderValue::from_str("application/json").unwrap());
            response.headers_mut().insert(
                "x-restate-server",
                HeaderValue::from_str("restate-sdk-rust/0.1.0").unwrap(),
            );
            Ok(response)
        }

        // Convert to uppercase before sending back to the client using a stream.
        (&Method::POST, "/services/Greeter/greet") => {
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });

            Ok(Response::new(frame_stream.boxed()))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            println!("{}, {}", req.method(), req.uri().path());
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}
