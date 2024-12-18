use crate::{
    connection::{Http2Receiver, Http2Sender, MessageReceiver, MessageSender, RestateStreamConsumer},
    context::{Context, ContextData, ContextInstance},
    endpoint::handler::handle_invocation,
};
use restate_sdk_core::ServiceHandler;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub async fn handle<Context, Func, Input, Output>(
    handler: Func,
    token: Option<CancellationToken>,
    receiver: Http2Receiver,
    sender: Http2Sender,
    test: bool,
) where
    for<'a> Input: Serialize + Deserialize<'a>,
    for<'a> Output: Serialize + Deserialize<'a>,
    Func: ServiceHandler<Context, Input, Output = Result<Output, anyhow::Error>> + Send + Sync + 'static,
    Context: ContextInstance,
{
    handle_invocation(handler, token, receiver, sender, test).await;
    debug!("Invocation task completed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::{setup_mock_connection, MessageSender, RestateStreamConsumer},
        context::{Context, ContextBase},
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
    use tracing::info;
    use tracing_test::traced_test;

    #[derive(Serialize, Deserialize)]
    pub struct ExecInput {
        name: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ExecOutput {
        status: String,
    }

    async fn service_fn(ctx: Context, input: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        let output = ctx
            .invoke(
                greet_fn,
                "Greeter".to_string(),
                "greet".to_string(),
                input,
                None,
                None,
            )
            .await
            .unwrap();
        Ok(ExecOutput {
            status: output.status,
        })
    }

    async fn greet_fn(ctx: Context, name: ExecInput) -> Result<ExecOutput, anyhow::Error> {
        Ok(ExecOutput { status: name.name })
    }

    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_handle_connection() {
        let (receiver, sender, mut output_rx) = setup_mock_connection(VecDeque::from([
            (
                None,
                MessageType::Start,
                ProtocolMessage::Start(restate_sdk_types::service_protocol::StartMessage {
                    id: Default::default(),
                    debug_id: "".to_string(),
                    known_entries: 2,
                    state_map: vec![],
                    partial_state: false,
                    key: "".to_string(),
                    retry_count_since_last_stored_entry: 0,
                    duration_since_last_stored_entry: 0,
                }),
            ),
            (
                None,
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
                None,
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
                        idempotency_key: None,
                        result: Some(call_entry_message::Result::Value("{\"status\":\"test\"}".into())),
                    }
                    .encode_to_vec()
                    .into(),
                )
                .into(),
            ),
        ]));

        let token = CancellationToken::new();
        let token2 = token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(10)) => {

                }
                _ = handle_invocation(service_fn, Some(token2), receiver, sender, true) => {

                }
            }
            info!("Invocation done");
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        break;
                    }
                    message = output_rx.recv() => {
                        if let Some(message) = message {
                            info!("Output message: {:?}", message);
                        }
                    }
                }
            }
            info!("Invocation output ---done----->");
        });

        tokio::time::sleep(Duration::from_secs(5)).await;
        token.cancel();
        handle.await.unwrap();
    }
}
