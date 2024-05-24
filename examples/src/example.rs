#![deny(warnings)]

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{server::conn::http2, service::service_fn, Method, Request, Response, Result, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use restate_sdk::connection::{Connection, Http2Connection};
use restate_sdk_types::{
    journal::raw::{PlainEntryHeader, RawEntry},
    service_protocol,
};
use restate_service_protocol::message::ProtocolMessage;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

async fn service(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, anyhow::Error>>> {
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
        },
        {
          "name": "greet2",
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

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(full(manifest).map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }

        // Convert to uppercase before sending back to the client using a stream.
        (&Method::POST, "/invoke/Greeter/greet2") => {
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }

            let (mut http2conn, boxed_body) = Http2Connection::new(req);

            tokio::spawn(async move {
                //tokio::time::sleep(Duration::from_secs(5)).await;
                let result = service_protocol::OutputEntryMessage {
                    name: "".to_string(),
                    result: Some(service_protocol::output_entry_message::Result::Value(
                        Bytes::from("success3"),
                    )),
                };
                http2conn.send(ProtocolMessage::UnparsedEntry(RawEntry::new(
                    PlainEntryHeader::Output,
                    result.encode_to_vec().into(),
                )));

                http2conn.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                tokio::time::sleep(Duration::from_secs(2)).await;
            });

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/restate")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();
            Ok(response)
        }

        (&Method::POST, "/invoke/Greeter/greet") => {
            println!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                println!("{:?}, {:?}", name, header);
            }

            let (mut http2conn, boxed_body) = Http2Connection::new(req);

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let result = service_protocol::CallEntryMessage {
                    service_name: "Greeter".to_string(),
                    handler_name: "greet2".to_string(),
                    parameter: Bytes::from("hello again"),
                    headers: vec![],
                    key: "".to_string(),
                    name: "".to_string(),
                    result: None,
                };

                http2conn.send(ProtocolMessage::UnparsedEntry(RawEntry::new(
                    PlainEntryHeader::Call {
                        is_completed: false,
                        enrichment_result: None,
                    },
                    result.encode_to_vec().into(),
                )));

                tokio::time::sleep(Duration::from_secs(10)).await;

                let result = service_protocol::OutputEntryMessage {
                    name: "".to_string(),
                    result: Some(service_protocol::output_entry_message::Result::Value(
                        Bytes::from("success2"),
                    )),
                };

                http2conn.send(ProtocolMessage::UnparsedEntry(RawEntry::new(
                    PlainEntryHeader::Output,
                    result.encode_to_vec().into(),
                )));

                http2conn.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                tokio::time::sleep(Duration::from_secs(10)).await;
            });

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/restate")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();

            Ok(response)
        }

        // Return the 404 Not Found for other routes.
        _ => {
            println!("{}, {}", req.method(), req.uri().path());
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty().map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}
