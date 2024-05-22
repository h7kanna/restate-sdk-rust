pub struct RestateEndpoint {}

mod http2_handler {
    use crate::{
        connection::{Connection, Http2Connection},
        invocation::InvocationBuilder,
        machine::StateMachine,
    };
    use parking_lot::Mutex;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    pub async fn handle(connection: Http2Connection) {
        // step 1: collect all journal entries
        let mut builder = InvocationBuilder::new();
        connection.stream_to_consumer(&mut builder).await;
        let invocation = builder.build();

        // step 2: create the state machine
        let (mut state_machine, mut suspension_rx) = StateMachine::new(invocation);
        connection.stream_to_consumer(&mut state_machine).await;

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
            connection::{Connection, Http2Connection},
            context::RestateContext,
        };
        use std::time::Duration;
        use tokio::sync::mpsc::channel;
        use tokio_util::sync::CancellationToken;
        use tracing_test::traced_test;

        #[traced_test]
        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_handle() {
            let token = CancellationToken::new();
            let token2 = token.clone();
            let input = "hello".to_string();

            let connection = Http2Connection {};

            let handle = tokio::spawn(async move {
                tokio::select! {
                    _ = token2.cancelled() => {

                    }
                    _ = handle(connection) => {

                    }
                }
            });

            tokio::time::sleep(Duration::from_secs(10)).await;

            token.cancel();

            handle.await.unwrap();
        }
    }
}
