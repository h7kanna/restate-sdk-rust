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
    sender: impl MessageSender + 'static,
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
    let token3 = token.clone();

    // step 3: create connection stream consumer
    tokio::spawn(async move {
        // Connection handler
        let message_consumer = message_consumer;
        let token = token2;
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break;
                }
                message = receiver.recv() => {
                   if let Some(message) = message {
                       let mut message_consumer = message_consumer.lock();
                        message_consumer.handle_message(message);
                    }
                }
            }
        }
        println!("Stream consumption completed");
    });

    // step 4: create suspension stream consumer
    tokio::spawn(async move {
        let suspension_consumer = suspension_consumer;
        let token = token3;
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break;
                }
                message = suspension_rx.recv() => {
                   if let Some(message) = message {
                       println!("scheduling suspension: {:?}", message);
                        suspension_consumer.lock().suspend();
                    }
                }
            }
        }
        println!("Suspension task completed");
    });

    // step 5: invoke the function
    StateMachine::invoke(handler, state_machine).await;
    token.cancel();
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
        name: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ExecOutput {
        status: String,
    }

    async fn service_fn(ctx: RestateContext, input: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        let output = ctx
            .invoke(greet_fn, "Greeter".to_string(), "greet".to_string(), input, None)
            .await
            .unwrap();
        Ok(ExecOutput {
            status: output.status,
        })
    }

    async fn greet_fn(ctx: RestateContext, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        Ok(ExecOutput { status: name.name })
    }

    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_handle_connection() {
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
                            value: "{\"name\":\"test\"}".into(),
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
                            parameter: "{\"name\":\"test\"}".into(),
                            headers: vec![],
                            name: "".to_string(),
                            key: "".to_string(),
                            result: Some(call_entry_message::Result::Value("{\"status\":\"test\"}".into())),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .into(),
                ),
            ]),
            output_messages: output_tx,
        };
        let connection2 = connection.clone();

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(10)) => {

                }
                _ = handle_invocation(service_fn, connection2.clone(), connection2) => {

                }
            }
            println!("Invocation done done");
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        break;
                    }
                    message = output_rx.recv() => {
                        if let Some(message) = message {
                            println!("Output message: {:?}", message);
                        }
                    }
                }
            }
            println!("Invocation ---dfasd----->");
        });

        drop(connection.output_messages);

        handle.await.unwrap();
    }
}
