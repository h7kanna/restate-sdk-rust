use crate::connection::RestateStreamConsumer;

pub struct RestateEndpoint {}

mod http2_handler {
    use crate::{
        connection::{Connection, Http2Connection, MessageStreamer},
        invocation::InvocationBuilder,
        machine::StateMachine,
    };
    use parking_lot::Mutex;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    pub async fn handle(connection: Http2Connection) {
        handle_connection(connection).await
    }

    pub async fn handle_connection(connection: impl Connection + MessageStreamer + 'static) {
        // step 1: collect all journal entries
        let mut builder = InvocationBuilder::new();
        connection.stream_to_consumer(&mut builder).await;
        let invocation = builder.build();

        // step 2: create the state machine
        let (mut state_machine, mut suspension_rx) = StateMachine::new(Box::new(connection), invocation);
        //connection.stream_to_consumer(&mut state_machine).await;

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
        StateMachine::invoke(state_machine).await
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{
            connection::{Connection, RestateStreamConsumer},
            context::RestateContext,
        };
        use restate_sdk_types::{
            protocol::{
                Message::{CallEntryMessage, InputEntryMessage, StartMessage},
                INPUT_ENTRY_MESSAGE_TYPE, START_MESSAGE_TYPE,
            },
            Message,
        };
        use std::time::Duration;
        use tokio::sync::mpsc::{channel, UnboundedSender};
        use tokio_util::sync::CancellationToken;
        use tracing_test::traced_test;

        struct TestDriver {
            input_messages: Vec<Message>,
            output_messages: UnboundedSender<Message>,
        }

        impl MessageStreamer for TestDriver {
            async fn stream_to_consumer(&self, mut consumer: impl RestateStreamConsumer) {
                for message in &self.input_messages {
                    consumer.handle(message.clone()).await;
                }
            }
        }

        impl Connection for TestDriver {
            fn send(&mut self, message: Message) {
                self.output_messages.send(message).unwrap();
            }
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
                    Message {
                        message_type: START_MESSAGE_TYPE,
                        message: StartMessage(1, restate_sdk_protos::StartMessage {
                            id: Default::default(),
                            debug_id: "".to_string(),
                            known_entries: 1,
                            state_map: vec![],
                            partial_state: false,
                            key: "".to_string(),
                        }),
                        completed: false,
                        requires_ack: None,
                    },
                    Message {
                        message_type: INPUT_ENTRY_MESSAGE_TYPE,
                        message: InputEntryMessage(1, restate_sdk_protos::InputEntryMessage {
                            headers: vec![],
                            value: Default::default(),
                            name: "".to_string(),
                        }),
                        completed: false,
                        requires_ack: None,
                    },
                ],
                output_messages: output_tx,
            };

            let handle = tokio::spawn(async move {
                tokio::select! {
                    _ = token2.cancelled() => {

                    }
                    _ = handle_connection(connection) => {

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
}
