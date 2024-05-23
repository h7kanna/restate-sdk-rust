use async_trait::async_trait;
use futures::{pin_mut, Stream};
use futures_util::StreamExt;
use restate_sdk_types::{protocol::Message::CallEntryMessage, Message};

pub trait MessageStreamer {
    async fn stream_to_consumer(&self, consumer: impl RestateStreamConsumer);
}

#[async_trait]
pub trait Connection: Send {
    fn send(&mut self, message: Message);
}

pub trait RestateStreamConsumer {
    async fn handle(&mut self, message: Message) -> bool;
}

pub struct Http2Connection {}

impl Http2Connection {
    pub async fn stream(&self, messages: impl Stream<Item = Message>) {
        pin_mut!(messages);

        while let Some(message) = messages.next().await {}
    }
}

impl MessageStreamer for Http2Connection {
    async fn stream_to_consumer(&self, mut consumer: impl RestateStreamConsumer) {
        consumer
            .handle(Message {
                message_type: 0,
                message: CallEntryMessage(1, restate_sdk_service_protocol::CallEntryMessage {
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
            })
            .await;
    }
}

impl Connection for Http2Connection {
    fn send(&mut self, message: Message) {
        todo!()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_connection() {}
}
