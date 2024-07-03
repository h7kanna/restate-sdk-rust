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
use tracing::info;

pub async fn handle_invocation<C, F, I, R>(
    handler: F,
    token: Option<CancellationToken>,
    mut receiver: impl MessageReceiver + 'static,
    sender: impl MessageSender + 'static,
    test: bool,
) where
    for<'a> I: Serialize + Deserialize<'a>,
    for<'a> R: Serialize + Deserialize<'a>,
    F: ServiceHandler<C, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
    C: ContextInstance,
{
    let token = token.unwrap_or_else(|| CancellationToken::new());

    // step 1: collect all journal entries
    let mut builder = InvocationBuilder::new();
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                info!("Invocation cancelled");
                return;
            }
            message = receiver.recv() => {
                if let Some(message) = message {
                    info!("Messages received {:?}", message);
                    if builder.handle_message(message) {
                        info!("Messages completed");
                        break;
                    }
                }
            }
        }
    }
    let invocation = builder.build();

    info!("Invocation started {:?}", invocation.id);
    let invocation_id = invocation.id.clone();
    // step 2: create the state machine
    let (state_machine, mut suspension_rx) = if test {
        StateMachine::new(Some(Box::new(sender)), None, invocation)
    } else {
        StateMachine::new(None, Some(Box::new(sender)), invocation)
    };

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
        info!("Stream consumption completed");
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
                       info!("scheduling suspension: {:?}", message);
                        suspension_consumer.lock().suspend();
                    }
                }
            }
        }
        info!("Suspension task completed");
    });

    // step 5: invoke the function
    StateMachine::invoke(token, handler, state_machine).await
}
