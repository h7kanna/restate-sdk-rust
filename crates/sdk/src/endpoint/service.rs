use crate::connection::{empty, full, setup_connection, MessageSender};
use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};
use prost::Message;
use restate_sdk_types::{
    journal::raw::{PlainEntryHeader, PlainRawEntry},
    service_protocol,
};
use restate_service_protocol::message::ProtocolMessage;
use std::time::Duration;
use tracing::info;

async fn service(
    req: Request<hyper::body::Incoming>,
) -> hyper::Result<Response<BoxBody<Bytes, anyhow::Error>>> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/discover") => {
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
            info!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                info!("{:?}, {:?}", name, header);
            }

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/vnd.restate.endpointmanifest.v1+json")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(full(manifest).map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }

        // Convert to uppercase before sending back to the client using a stream.
        (&Method::POST, "/invoke/Greeter/greet2") => {
            info!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                info!("{:?}, {:?}", name, header);
            }

            let (_, sender, boxed_body) = setup_connection(req);

            tokio::spawn(async move {
                //tokio::time::sleep(Duration::from_secs(5)).await;
                sender.send(
                    PlainRawEntry::new(
                        PlainEntryHeader::Output,
                        service_protocol::OutputEntryMessage {
                            name: "".to_string(),
                            result: Some(service_protocol::output_entry_message::Result::Value(
                                Bytes::from("success3"),
                            )),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                );

                sender.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                tokio::time::sleep(Duration::from_secs(2)).await;
            });

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/vnd.restate.invocation.v1")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();
            Ok(response)
        }

        (&Method::POST, "/invoke/Greeter/greet") => {
            info!("{}, {}", req.method(), req.uri().path());
            for (name, header) in req.headers() {
                info!("{:?}, {:?}", name, header);
            }

            let (_, sender, boxed_body) = setup_connection(req);

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;

                sender.send(
                    PlainRawEntry::new(
                        PlainEntryHeader::Call {
                            is_completed: false,
                            enrichment_result: None,
                        },
                        service_protocol::CallEntryMessage {
                            service_name: "Greeter".to_string(),
                            handler_name: "greet2".to_string(),
                            parameter: Bytes::from("hello again"),
                            headers: vec![],
                            key: "".to_string(),
                            name: "".to_string(),
                            result: None,
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                );

                tokio::time::sleep(Duration::from_secs(10)).await;

                sender.send(
                    PlainRawEntry::new(
                        PlainEntryHeader::Output,
                        service_protocol::OutputEntryMessage {
                            name: "".to_string(),
                            result: Some(service_protocol::output_entry_message::Result::Value(
                                Bytes::from("success2"),
                            )),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                );

                sender.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                tokio::time::sleep(Duration::from_secs(10)).await;
            });

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/vnd.restate.invocation.v1")
                .header("x-restate-server", "restate-sdk-rust/0.1.0")
                .body(boxed_body)
                .unwrap();

            Ok(response)
        }

        // Return the 404 Not Found for other routes.
        _ => {
            info!("{}, {}", req.method(), req.uri().path());
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty().map_err(|e| e.into()).boxed())
                .unwrap();
            Ok(response)
        }
    }
}
