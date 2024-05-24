use crate::{
    connection::{Connection, Http2Connection, MessageStreamer},
    context::RestateContext,
    invocation::InvocationBuilder,
    machine::StateMachine,
};
use parking_lot::Mutex;
use restate_sdk_core::ServiceHandler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn handle<F, I, R>(service_fn: F, connection: Http2Connection)
where
    for<'a> I: Serialize + Deserialize<'a>,
    for<'a> R: Serialize + Deserialize<'a>,
    F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
{
    handle_invocation(service_fn, connection).await
}

pub async fn handle_invocation<F, I, R>(
    service_fn: F,
    mut connection: impl Connection + MessageStreamer + 'static,
) where
    for<'a> I: Serialize + Deserialize<'a>,
    for<'a> R: Serialize + Deserialize<'a>,
    F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
{
    // step 1: collect all journal entries
    let mut builder = InvocationBuilder::new();
    connection.pipe_to_consumer(&mut builder).await;
    let invocation = builder.build();

    // step 2: create the state machine
    let (mut state_machine, mut suspension_rx) = StateMachine::new(Box::new(connection), invocation);
    //connection.pipe_to_consumer(&mut state_machine).await;

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
        tokio::select! {
            _ = token.cancelled() => {

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
    StateMachine::invoke(service_fn, state_machine).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::{Connection, RestateStreamConsumer},
        context::RestateContext,
    };
    use prost::Message;
    use restate_sdk_types::{
        journal::raw::{PlainEntryHeader, PlainRawEntry},
        service_protocol::call_entry_message,
    };
    use restate_service_protocol::message::{MessageType, ProtocolMessage};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use tokio::sync::mpsc::{channel, UnboundedSender};
    use tokio_util::sync::CancellationToken;
    use tracing_test::traced_test;

    struct TestDriver {
        input_messages: Vec<(MessageType, ProtocolMessage)>,
        output_messages: UnboundedSender<ProtocolMessage>,
    }

    impl MessageStreamer for TestDriver {
        async fn pipe_to_consumer(&mut self, mut consumer: impl RestateStreamConsumer) {
            for message in &self.input_messages {
                consumer.handle(message.clone());
            }
        }
    }

    impl Connection for TestDriver {
        fn send(&mut self, message: ProtocolMessage) {
            self.output_messages.send(message).unwrap();
        }
    }

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
            input_messages: vec![
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
            ],
            output_messages: output_tx,
        };

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = token2.cancelled() => {

                }
                _ = handle_invocation(service_fn,connection) => {

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
