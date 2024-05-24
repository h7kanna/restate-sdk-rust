use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{pin_mut, Stream};
use futures_util::{StreamExt, TryStreamExt};
use http::Request;
use http_body::Frame;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use prost::Message;
use restate_sdk_types::{
    protocol,
    service_protocol::{EndMessage, ServiceProtocolVersion},
};
use restate_service_protocol::message::{Decoder, Encoder, ProtocolMessage};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub trait RestateStreamConsumer: Send {
    fn handle(&mut self, message: restate_sdk_types::Message) -> bool;
}

pub trait MessageStreamer: Send {
    async fn pipe_to_consumer(&mut self, consumer: impl RestateStreamConsumer);
}

#[async_trait]
pub trait Connection: Send {
    fn send(&mut self, message: ProtocolMessage);
}

pub struct Http2Connection {
    inbound_rx: UnboundedReceiver<ProtocolMessage>,
    outbound_tx: UnboundedSender<ProtocolMessage>,
}

impl Http2Connection {
    pub fn new(request: Request<hyper::body::Incoming>) -> (Self, BoxBody<Bytes, anyhow::Error>) {
        // Setup inbound message buffer
        let frame_stream = http_body_util::BodyStream::new(
            request
                .into_body()
                .map_frame(move |frame| frame)
                .map_err(|_e| anyhow!("error"))
                .boxed(),
        );

        let (inbound_tx, mut inbound_rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
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

        // Setup outbound message buffer
        let (outbound_tx, outbound_rx) = tokio::sync::mpsc::unbounded_channel();
        let encoder = Encoder::new(ServiceProtocolVersion::V1);
        let boxed_body = BodyExt::boxed(StreamBody::new(UnboundedReceiverStream::new(outbound_rx).map(
            move |message| {
                let result = encoder.encode(message);
                Ok(Frame::data(result))
            },
        )));

        (
            Self {
                inbound_rx,
                outbound_tx,
            },
            boxed_body,
        )
    }
}

impl MessageStreamer for Http2Connection {
    async fn pipe_to_consumer(&mut self, mut consumer: impl RestateStreamConsumer) {
        // Setup inbound message consumer
        loop {
            if let Some(message) = self.inbound_rx.recv().await {
                let message = restate_sdk_types::Message {
                    message_type: 0,
                    message: protocol::Message::EndMessage(1, EndMessage {}),
                    completed: false,
                    requires_ack: None,
                };
                if !consumer.handle(message) {
                    continue;
                }
            }
        }
    }
}

impl Connection for Http2Connection {
    fn send(&mut self, message: ProtocolMessage) {
        if let Err(err) = self.outbound_tx.send(message) {
            println!("Outbound send error: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_connection() {}
}
