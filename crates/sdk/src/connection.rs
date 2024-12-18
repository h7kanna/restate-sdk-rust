use anyhow::anyhow;
use bytes::Bytes;
use futures::{pin_mut, Stream};
use futures_util::{StreamExt, TryStreamExt};
use http::Request;
use http_body::Frame;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full, StreamBody};
use prost::Message;
use restate_sdk_types::service_protocol::ServiceProtocolVersion;
use restate_service_protocol::message::{Decoder, Encoder, MessageType, ProtocolMessage};
use std::{collections::VecDeque, future::Future};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::debug;

pub(crate) trait Sealed {}

pub trait MessageReceiver: Sealed + Send {
    fn recv(&mut self) -> impl Future<Output = Option<(MessageType, ProtocolMessage)>> + Send;
}

pub trait MessageSender: Sealed + Send {
    fn send(&self, message: ProtocolMessage);
}

pub trait RestateStreamConsumer {
    fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool;
}

pub struct MockHttp2Receiver {
    inbound_rx: VecDeque<(Option<String>, MessageType, ProtocolMessage)>,
}

pub struct MockHttp2Sender {
    outbound_tx: UnboundedSender<ProtocolMessage>,
}

impl Sealed for MockHttp2Receiver {}

impl MessageReceiver for MockHttp2Receiver {
    async fn recv(&mut self) -> Option<(MessageType, ProtocolMessage)> {
        self.inbound_rx.pop_front().map(|message| (message.1, message.2))
    }
}

impl Sealed for MockHttp2Sender {}

impl MessageSender for MockHttp2Sender {
    fn send(&self, message: ProtocolMessage) {
        // Ignore the mock output
        let _ = self.outbound_tx.send(message);
    }
}

pub fn setup_mock_connection(
    inbound_rx: VecDeque<(Option<String>, MessageType, ProtocolMessage)>,
) -> (
    MockHttp2Receiver,
    MockHttp2Sender,
    UnboundedReceiver<ProtocolMessage>,
) {
    let (outbound_tx, outbound_rx) = tokio::sync::mpsc::unbounded_channel();
    (
        MockHttp2Receiver { inbound_rx },
        MockHttp2Sender { outbound_tx },
        outbound_rx,
    )
}

pub struct Http2Receiver {
    inbound_rx: UnboundedReceiver<(MessageType, ProtocolMessage)>,
}

pub struct Http2Sender {
    outbound_tx: UnboundedSender<ProtocolMessage>,
}

impl Sealed for Http2Receiver {}

impl MessageReceiver for Http2Receiver {
    async fn recv(&mut self) -> Option<(MessageType, ProtocolMessage)> {
        self.inbound_rx.recv().await
    }
}

impl Sealed for Http2Sender {}

impl MessageSender for Http2Sender {
    fn send(&self, message: ProtocolMessage) {
        if let Err(err) = self.outbound_tx.send(message) {
            debug!("Outbound send error: {}", err);
        }
    }
}

pub fn setup_connection(
    request: Request<hyper::body::Incoming>,
) -> (Http2Receiver, Http2Sender, BoxBody<Bytes, anyhow::Error>) {
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
                            //info!("Header: {:?}, Message: {:?}", header, message);
                            if let Err(err) = inbound_tx.send((header.message_type(), message)) {
                                debug!("Send failed {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        debug!("decode error: {:?}", err);
                    }
                }
            };
        }
        debug!("HTTP request stream closed");
    });

    // Setup outbound message buffer
    let (outbound_tx, outbound_rx) = tokio::sync::mpsc::unbounded_channel();
    let encoder = Encoder::new(ServiceProtocolVersion::V1);
    let boxed_body = BodyExt::boxed(StreamBody::new(UnboundedReceiverStream::new(outbound_rx).map(
        move |message| {
            debug!("Sending response message: {:?}", message);
            let result = encoder.encode(message);
            Ok(Frame::data(result))
        },
    )));

    (
        Http2Receiver { inbound_rx },
        Http2Sender { outbound_tx },
        boxed_body,
    )
}

pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(|never| match never {}).boxed()
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into()).map_err(|never| match never {}).boxed()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_connection() {}
}
