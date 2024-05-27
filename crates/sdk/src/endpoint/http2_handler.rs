use crate::{
    connection::{Http2Receiver, Http2Sender, MessageReceiver, MessageSender, RestateStreamConsumer},
    context::RestateContext,
    invocation::InvocationBuilder,
    machine::StateMachine,
};
use parking_lot::Mutex;
use restate_sdk_core::ServiceHandler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn handle<F, I, R>(handler: F, receiver: Http2Receiver, sender: Http2Sender)
where
    for<'a> I: Serialize + Deserialize<'a>,
    for<'a> R: Serialize + Deserialize<'a>,
    F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
{
    handle_invocation(handler, receiver, sender).await
}

pub async fn handle_invocation<F, I, R>(
    handler: F,
    mut receiver: impl MessageReceiver + 'static,
    mut sender: impl MessageSender + 'static,
) where
    for<'a> I: Serialize + Deserialize<'a>,
    for<'a> R: Serialize + Deserialize<'a>,
    F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
{
    // step 1: collect all journal entries
    let mut builder = InvocationBuilder::new();
    loop {
        if let Some(message) = receiver.recv().await {
            if builder.handle_message(message) {
                break;
            }
        }
    }
    let invocation = builder.build();

    // step 2: create the state machine
    let (state_machine, mut suspension_rx) = StateMachine::new(Box::new(sender), invocation);

    let state_machine = Arc::new(Mutex::new(state_machine));
    let message_consumer = state_machine.clone();
    let suspension_consumer = state_machine.clone();

    let token = CancellationToken::new();
    let token2 = token.clone();

    // step 3: create connection stream consumer
    tokio::spawn(async move {
        // Connection handler
        let message_consumer = message_consumer;
        let token = token2;
        loop {
            tokio::select! {
                _ = token.cancelled() => {

                }
                message = receiver.recv() => {
                   if let Some(message) = message {
                       let mut message_consumer = message_consumer.lock();
                        println!("Stream consumption completed");
                        message_consumer.handle_message(message);
                    }
                }
            }
        }
    });

    // step 4: create suspension stream consumer
    tokio::spawn(async move {
        let suspension_consumer = suspension_consumer;
        while let Some(message) = suspension_rx.recv().await {
            println!("scheduling suspension: {:?}", message);
            suspension_consumer.lock().suspend();
        }
    });

    // step 5: invoke the function
    StateMachine::invoke(handler, state_machine).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::{MessageSender, RestateStreamConsumer},
        context::RestateContext,
    };
    use prost::Message;
    use restate_sdk_types::{
        journal::raw::{PlainEntryHeader, PlainRawEntry},
        service_protocol::call_entry_message,
    };
    use restate_service_protocol::message::{MessageType, ProtocolMessage};
    use serde::{Deserialize, Serialize};
    use std::{collections::VecDeque, time::Duration};
    use tokio::sync::mpsc::{channel, UnboundedSender};
    use tokio_util::sync::CancellationToken;
    use tracing_test::traced_test;

    #[derive(Clone)]
    struct TestDriver {
        input_messages: VecDeque<(MessageType, ProtocolMessage)>,
        output_messages: UnboundedSender<ProtocolMessage>,
    }

    impl MessageReceiver for TestDriver {
        async fn recv(&mut self) -> Option<(MessageType, ProtocolMessage)> {
            self.input_messages.pop_front()
        }
    }

    impl MessageSender for TestDriver {
        fn send(&self, message: ProtocolMessage) {
            self.output_messages.send(message).unwrap();
        }
    }

    impl crate::connection::Sealed for TestDriver {}

    #[derive(Serialize, Deserialize)]
    pub struct ExecInput {
        test: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ExecOutput {
        test: String,
    }

    async fn service_fn(ctx: RestateContext, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        let output = ctx
            .invoke(greet_fn, "Greeter".to_string(), "greet".to_string(), name, None)
            .await
            .unwrap();
        Ok(ExecOutput { test: output.test })
    }

    async fn greet_fn(ctx: RestateContext, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        Ok(ExecOutput { test: name.test })
    }

    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_handle_connection() {
        let token = CancellationToken::new();
        let token2 = token.clone();
        let input = "hello".to_string();

        let (output_tx, mut output_rx) = tokio::sync::mpsc::unbounded_channel();
        let connection = TestDriver {
            input_messages: VecDeque::from([
                (
                    MessageType::Start,
                    ProtocolMessage::Start(restate_sdk_types::service_protocol::StartMessage {
                        id: Default::default(),
                        debug_id: "".to_string(),
                        known_entries: 2,
                        state_map: vec![],
                        partial_state: false,
                        key: "".to_string(),
                    }),
                ),
                (
                    MessageType::InputEntry,
                    PlainRawEntry::new(
                        PlainEntryHeader::Input,
                        restate_sdk_types::service_protocol::InputEntryMessage {
                            headers: vec![],
                            value: "{\"test\":\"test\"}".into(),
                            name: "".to_string(),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                ),
                (
                    MessageType::InvokeEntry,
                    PlainRawEntry::new(
                        PlainEntryHeader::Call {
                            is_completed: false,
                            enrichment_result: None,
                        },
                        restate_sdk_types::service_protocol::CallEntryMessage {
                            service_name: "".to_string(),
                            handler_name: "".to_string(),
                            parameter: "{\"test\":\"test\"}".into(),
                            headers: vec![],
                            name: "".to_string(),
                            key: "".to_string(),
                            result: Some(call_entry_message::Result::Value("{\"test\":\"harsha\"}".into())),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                ),
            ]),
            output_messages: output_tx,
        };

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = token2.cancelled() => {

                }
                _ = handle_invocation(service_fn, connection.clone(), connection.clone()) => {

                }
            }
        });

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(2)) => {
                    break;
                }
                message = output_rx.recv() => {
                    if let Some(message) = message {
                        println!("{:?}", message);
                    }
                }
            }
        }

        token.cancel();

        handle.await.unwrap();
    }
}
