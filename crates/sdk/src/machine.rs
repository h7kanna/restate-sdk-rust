use crate::{
    connection::{MessageSender, RestateStreamConsumer},
    context::{Context, Request},
    invocation::Invocation,
    journal::Journal,
    logger::Logger,
    store::LocalStore,
};
use bytes::Bytes;
use parking_lot::{Mutex, MutexGuard};
use prost::Message;
use restate_sdk_core::ServiceHandler;
use restate_sdk_types::{
    endpoint_manifest::ProtocolMode,
    journal::{
        raw::{PlainEntryHeader, PlainRawEntry},
        Entry, EntryResult,
    },
    service_protocol,
    service_protocol::{run_entry_message, Failure},
};
use restate_service_protocol::message::{MessageType, ProtocolMessage};
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc, task::Waker};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

const SUSPENSION_MILLIS: u32 = 30000;

pub(crate) struct StateMachine {
    journal: Journal,
    machine_closed: bool,
    input_channel_closed: bool,
    local_state_store: Option<LocalStore>,
    logger: Logger,
    suspension_tx: UnboundedSender<String>,
    connection: Option<Box<dyn MessageSender>>,
    abort: Option<Box<dyn MessageSender>>,
    protocol_mode: ProtocolMode,
    input: Option<Bytes>,
}

impl StateMachine {
    pub fn new(
        abort: Option<Box<dyn MessageSender>>,
        connection: Option<Box<dyn MessageSender>>,
        invocation: Invocation,
    ) -> (Self, UnboundedReceiver<String>) {
        let input = invocation.invocation_value.clone();
        let (suspension_tx, suspension_rx) = unbounded_channel();
        (
            Self {
                journal: Journal::new(invocation),
                machine_closed: false,
                input_channel_closed: false,
                local_state_store: None,
                logger: Logger::new(),
                suspension_tx,
                connection,
                abort,
                protocol_mode: ProtocolMode::BidiStream,
                input,
            },
            suspension_rx,
        )
    }

    pub async fn invoke<F, I, R>(
        token: CancellationToken,
        handler: F,
        state_machine: Arc<Mutex<StateMachine>>,
    ) where
        for<'a> I: Serialize + Deserialize<'a>,
        for<'a> R: Serialize + Deserialize<'a>,
        F: ServiceHandler<Context, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
    {
        let input = state_machine.lock().input.clone().unwrap();
        let input = serde_json::from_slice(&input.to_vec()).unwrap();
        let request = Request {
            id: state_machine.lock().journal.invocation().id.clone(),
        };
        let ctx = Context::new(request, state_machine.clone());
        tokio::select! {
            _ = token.cancelled() => {
               println!("State machine cancelled");
            }
            result = handler(ctx, input) => {
                match result {
                    Ok(result) => {
                        let result = serde_json::to_string(&result).unwrap();
                        let output: ProtocolMessage = PlainRawEntry::new(
                            PlainEntryHeader::Output,
                            service_protocol::OutputEntryMessage {
                                name: "".to_string(),
                                result: Some(service_protocol::output_entry_message::Result::Value(
                                    result.into(),
                                )),
                            }
                            .encode_to_vec()
                            .into(),
                        )
                        .into();
                        let mut state_machine = state_machine.lock();
                        println!("Invocation output:  {:?}", output);
                        state_machine.send(output);
                        println!("Invocation end");
                        state_machine.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                    }
                    Err(err) => {
                        let error: ProtocolMessage = ProtocolMessage::Error(service_protocol::ErrorMessage {
                            code: 0,
                            message: "".to_string(),
                            description: err.to_string(),
                            related_entry_index: None,
                            related_entry_name: None,
                            related_entry_type: None,
                        });
                        let mut state_machine = state_machine.lock();
                        state_machine.send(error);
                        state_machine.send(ProtocolMessage::End(service_protocol::EndMessage {}));
                    }
                };
            }
        }
    }

    pub fn handle_user_code_message(
        &mut self,
        entry_index: u32,
        message: Entry,
        waker: Waker,
    ) -> Option<Bytes> {
        if self.machine_closed {
            // Return fused
        }
        let result = self
            .journal
            .handle_user_code_message(entry_index, message.clone(), waker);
        if result.is_none() {
            match &message {
                Entry::Input(_) => {}
                Entry::Output(_) => {}
                Entry::GetState(_) => {}
                Entry::SetState(_) => {}
                Entry::ClearState(_) => {}
                Entry::GetStateKeys(_) => {}
                Entry::ClearAllState => {}
                Entry::GetPromise(_) => {}
                Entry::PeekPromise(_) => {}
                Entry::CompletePromise(_) => {}
                Entry::Sleep(sleep) => {
                    println!(
                        "Result does not exist for entry index {:?}, sending sleep message",
                        entry_index
                    );
                    self.send(
                        PlainRawEntry::new(
                            PlainEntryHeader::Sleep { is_completed: false },
                            service_protocol::SleepEntryMessage {
                                wake_up_time: sleep.wake_up_time,
                                name: "".to_string(),
                                result: None,
                            }
                            .encode_to_vec()
                            .into(),
                        )
                        .into(),
                    );
                }
                Entry::Call(call) => {
                    self.send(
                        PlainRawEntry::new(
                            PlainEntryHeader::Call {
                                is_completed: false,
                                enrichment_result: None,
                            },
                            service_protocol::CallEntryMessage {
                                service_name: call.request.service_name.to_string(),
                                handler_name: call.request.handler_name.to_string(),
                                parameter: call.request.parameter.clone(),
                                headers: vec![],
                                key: call.request.key.to_string(),
                                name: "".to_string(),
                                result: None,
                            }
                            .encode_to_vec()
                            .into(),
                        )
                        .into(),
                    );
                }
                Entry::OneWayCall(one_way) => {
                    self.send(
                        PlainRawEntry::new(
                            PlainEntryHeader::OneWayCall {
                                enrichment_result: (),
                            },
                            service_protocol::OneWayCallEntryMessage {
                                service_name: one_way.request.service_name.to_string(),
                                handler_name: one_way.request.handler_name.to_string(),
                                parameter: one_way.request.parameter.clone(),
                                invoke_time: one_way.invoke_time,
                                headers: vec![],
                                key: one_way.request.key.to_string(),
                                name: "".to_string(),
                            }
                            .encode_to_vec()
                            .into(),
                        )
                        .into(),
                    );
                }
                Entry::Awakeable(_) => {}
                Entry::CompleteAwakeable(_) => {}
                Entry::Run(run) => {
                    let result = match &run.result {
                        EntryResult::Success(value) => run_entry_message::Result::Value(value.clone()),
                        EntryResult::Failure(code, message) => run_entry_message::Result::Failure(Failure {
                            code: (*code).into(),
                            message: message.to_string(),
                        }),
                    };
                    self.send(
                        PlainRawEntry::new(
                            PlainEntryHeader::Run,
                            service_protocol::RunEntryMessage {
                                name: "".to_string(),
                                result: Some(result),
                            }
                            .encode_to_vec()
                            .into(),
                        )
                        .into(),
                    );
                }
                Entry::Custom(_) => {}
            }
            None
        } else {
            println!(
                "Result exists for entry index {:?}, result: {:?}",
                entry_index, result
            );
            result
        }
    }

    pub fn get_next_user_code_journal_index(&self) -> u32 {
        return self.journal.get_next_user_code_journal_index();
    }

    fn send(&mut self, message: ProtocolMessage) {
        // If in processing or no use calls are performed at all
        if self.journal.is_processing() || self.journal.get_user_code_journal_index() == 0 {
            if let Some(ref connection) = self.connection {
                connection.send(message);
            } else if let Some(ref abort) = self.abort {
                abort.send(message);
            }
        }
    }

    fn hit_suspension(&self) {
        self.suspension_tx.send("suspend".to_string()).unwrap()
    }

    pub fn suspend(&self) {}
}

impl RestateStreamConsumer for MutexGuard<'_, StateMachine> {
    fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
        println!("Machine runtime message handler: {:?}", message);
        if self.machine_closed {
            return false;
        }
        if message.0 == MessageType::Completion {
            if let ProtocolMessage::Completion(message) = message.1 {
                self.journal.handle_runtime_completion_message(message);
            } else {
                // Wrong message type
            }
        } else if message.0 == MessageType::EntryAck {
            if let ProtocolMessage::EntryAck(message) = message.1 {
                self.journal.handle_runtime_entry_ack_message(message);
            } else {
                // Wrong message type
            }
        }
        // Clear suspension tasks
        false
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_state_machine() {}
}
