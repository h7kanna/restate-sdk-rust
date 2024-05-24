use anyhow::{anyhow, Error};
use async_trait::async_trait;
use bytes::Bytes;
use futures::{pin_mut, Stream};
use futures_util::StreamExt;
use http::Request;
use http_body::Frame;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use prost::Message;
use restate_sdk_types::{
    journal::raw::{PlainEntryHeader, RawEntry},
    protocol::Message::CallEntryMessage,
    service_protocol,
    service_protocol::ServiceProtocolVersion,
};
use restate_service_protocol::message::{Decoder, Encoder, ProtocolMessage};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::mpsc::error::SendError;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub trait MessageStreamer: Send {
    fn pipe_to_consumer(&self, consumer: impl RestateStreamConsumer);
}

#[async_trait]
pub trait Connection: Send {
    fn send(&mut self, message: restate_sdk_types::Message);
}

pub trait RestateStreamConsumer :Send {
    fn handle(&mut self, message: restate_sdk_types::Message) -> bool;
}

pub struct Http2Connection {
    inbound_rx: UnboundedReceiver<ProtocolMessage>,
    outbound_tx: UnboundedSender<ProtocolMessage>,
    consumer: Option<Box<dyn RestateStreamConsumer>>,
}

impl Http2Connection {
    pub fn new(
        request: Request<hyper::body::Incoming>,
    ) -> (Self, BoxBody<Bytes, anyhow::Error>) {
        let frame_stream = http_body_util::BodyStream::new(
            request
                .into_body()
                .map_frame(move |frame| frame)
                .map_err(|_e| anyhow!("error"))
                .boxed(),
        );

        let (inbound_tx, mut inbound_rx) = tokio::sync::mpsc::unbounded_channel();
        let decoder_task = tokio::spawn(async move {
            let mut decoder = Decoder::new(ServiceProtocolVersion::V1, usize::MAX, None);
            pin_mut!(frame_stream);
            while let Some(Ok(frame)) = frame_stream.next().await {
                if let Ok(data) = frame.into_data() {
                    decoder.push(data);
                    match decoder.consume_next() {
                        Ok(result) => {
                            if let Some((header, message)) = result {
                                println!("Header: {:?}, Message: {:?}", header, message);
                                if let Err(err) = inbound_tx.send(message) {
                                    println!("Send failed {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("decode error: {:?}", err);
                        }
                    }
                };
            }
        });

        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::unbounded_channel();
        let (encoded_message_tx, encoded_message_rx) = tokio::sync::mpsc::unbounded_channel();
        let encoded_message_stream = UnboundedReceiverStream::new(encoded_message_rx);
        let stream_body = StreamBody::new(encoded_message_stream);
        let boxed_body = BodyExt::boxed(stream_body);

        let encoder_task = tokio::spawn(async move {
            let encoder = Encoder::new(ServiceProtocolVersion::V1);
            while let Some(m) = outbound_rx.recv().await {
                let result = encoder.encode(m);
                 if let Err(err) = encoded_message_tx.send(Ok(Frame::data(result))) {
                    println!("Send failed {}", err);
                }
            }
        });

        (
            Self {
                outbound_tx,
                inbound_rx,
                consumer: None,
            },
            boxed_body,
        )
    }

    pub async fn stream(&self, messages: impl Stream<Item = restate_sdk_types::Message>) {
        pin_mut!(messages);

        while let Some(message) = messages.next().await {}
    }
}

impl MessageStreamer for Http2Connection {
    fn pipe_to_consumer(&self, mut consumer: impl RestateStreamConsumer) {
        consumer.handle(restate_sdk_types::Message {
            message_type: 0,
            message: CallEntryMessage(1, restate_sdk_types::service_protocol::CallEntryMessage {
                service_name: "".to_string(),
                handler_name: "".to_string(),
                parameter: Default::default(),
                headers: vec![],
                key: "".to_string(),
                name: "".to_string(),
                result: None,
            }),
            completed: false,
            requires_ack: None,
        });
    }
}

impl Connection for Http2Connection {
    fn send(&mut self, message: restate_sdk_types::Message) {
        todo!()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_connection() {}
}
