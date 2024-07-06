use crate::{
    connection::{MessageReceiver, MessageSender, RestateStreamConsumer},
    context::ContextInstance,
    invocation::InvocationBuilder,
    machine::StateMachine,
};
use parking_lot::Mutex;
use restate_sdk_core::ServiceHandler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub async fn handle_invocation<Context, Func, Input, Output>(
    handler: Func,
    token: Option<CancellationToken>,
    mut receiver: impl MessageReceiver + 'static,
    sender: impl MessageSender + 'static,
    test: bool,
) where
    for<'a> Input: Serialize + Deserialize<'a>,
    for<'a> Output: Serialize + Deserialize<'a>,
    Func: ServiceHandler<Context, Input, Output = Result<Output, anyhow::Error>> + Send + Sync + 'static,
    Context: ContextInstance,
{
    let token = token.unwrap_or_else(|| CancellationToken::new());

    // step 1: collect all journal entries
    let mut builder = InvocationBuilder::new();
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                debug!("Invocation cancelled");
                return;
            }
            message = receiver.recv() => {
                if let Some(message) = message {
                    debug!("Messages received {:?}", message);
                    if builder.handle_message(message) {
                        break;
                    }
                }
            }
        }
    }
    let invocation = builder.build();
    debug!("Invocation build completed {:?}", invocation.debug_id);
    debug!("Invocation machine started {:?}", invocation.debug_id);
    let invocation_id = invocation.id.clone();
    // step 2: create the state machine
    let (state_machine, mut suspension_rx) = StateMachine::new(test, Some(Box::new(sender)), invocation);
    let state_machine = Arc::new(Mutex::new(state_machine));
    let message_consumer = state_machine.clone();
    let suspension_consumer = state_machine.clone();

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
        debug!("Stream consumption completed");
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
                       debug!("scheduling suspension: {:?}", message);
                        suspension_consumer.lock().suspend();
                    }
                }
            }
        }
        debug!("Suspension task completed");
    });

    // step 5: invoke the function
    StateMachine::invoke(token, handler, state_machine).await
}
