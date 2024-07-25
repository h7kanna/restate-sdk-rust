use crate::{
    connection::{MessageSender, RestateStreamConsumer},
    context::{ContextData, ContextInstance, Request},
    invocation::Invocation,
    journal::Journal,
    logger::ReplayFilter,
    store::LocalStateStore,
};
use bytes::Bytes;
use futures::channel::oneshot;
use parking_lot::{Mutex, MutexGuard};
use prost::Message;
use restate_sdk_core::ServiceHandler;
use restate_sdk_types::{
    endpoint_manifest::ProtocolMode,
    journal::{
        raw::{PlainEntryHeader, PlainRawEntry},
        Entry, EntryResult, GetStateKeysResult, OutputEntry,
    },
    service_protocol,
    service_protocol::{
        complete_awakeable_entry_message, complete_promise_entry_message, get_state_keys_entry_message,
        run_entry_message, Failure,
    },
};
use restate_service_protocol::message::{MessageType, ProtocolMessage};
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc, task::Waker};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use tracing::{debug, field, info_span, Instrument};

const SUSPENSION_MILLIS: u32 = 30000;

pub(crate) struct StateMachine {
    journal: Journal,
    machine_closed: bool,
    input_channel_closed: bool,
    local_state_store: LocalStateStore,
    logger: ReplayFilter,
    suspension_tx: UnboundedSender<String>,
    connection: Option<Box<dyn MessageSender>>,
    abort_tx: Option<oneshot::Sender<bool>>,
    abort_on_replay: bool,
    protocol_mode: ProtocolMode,
    input: Option<Bytes>,
    span_replay_flag: bool,
}

impl StateMachine {
    pub fn new(
        abort_on_replay: bool,
        connection: Option<Box<dyn MessageSender>>,
        mut invocation: Invocation,
    ) -> (Self, UnboundedReceiver<String>) {
        let input = invocation.invocation_value.clone();
        let store = invocation.local_state_store.take();
        let (suspension_tx, suspension_rx) = unbounded_channel();
        (
            Self {
                journal: Journal::new(invocation),
                machine_closed: false,
                input_channel_closed: false,
                local_state_store: store.unwrap(),
                logger: ReplayFilter::new(),
                suspension_tx,
                connection,
                abort_tx: None,
                abort_on_replay,
                protocol_mode: ProtocolMode::BidiStream,
                input,
                span_replay_flag: true,
            },
            suspension_rx,
        )
    }

    pub fn local_state_store(&mut self) -> &mut LocalStateStore {
        &mut self.local_state_store
    }

    pub fn abort_on_replay(&mut self) {
        if self.abort_on_replay {
            let abort_tx = self.abort_tx.take();
            if let Some(abort_tx) = abort_tx {
                abort_tx.send(true).unwrap();
            }
        }
    }

    pub async fn invoke<Context, Func, Input, Output>(
        token: CancellationToken,
        handler: Func,
        state_machine: Arc<Mutex<StateMachine>>,
    ) where
        for<'a> Input: Serialize + Deserialize<'a>,
        for<'a> Output: Serialize + Deserialize<'a>,
        Func: ServiceHandler<Context, Input, Output = Result<Output, anyhow::Error>> + Send + Sync + 'static,
        Context: ContextInstance,
    {
        let input = state_machine.lock().input.clone().unwrap();
        let input = serde_json::from_slice(&input.to_vec()).unwrap();
        let id = state_machine.lock().journal.invocation().id.clone();
        let debug_id = state_machine
            .lock()
            .journal
            .invocation()
            .debug_id
            .clone()
            .unwrap();
        let span = info_span!(
            "invoke",
            "otel.name" = debug_id,
            "otel.kind" = "server",
            "replay" = field::Empty,
        );
        let request = Request { id };
        let (abort_tx, abort_rx) = oneshot::channel::<bool>();
        state_machine.lock().abort_tx = Some(abort_tx);
        let ctx = Context::new(request, state_machine.clone());
        let handle = handler(ctx, input).instrument(span);
        tokio::select! {
            _ = token.cancelled() => {
               debug!("State machine cancelled");
            }
            _ = abort_rx => {
                debug!("Invocation aborted");
            }
            result = handle => {
                match result {
                    Ok(result) => {
                        let result = serde_json::to_string(&result).unwrap();
                        let mut state_machine = state_machine.lock();
                         state_machine.handle_user_code_message(
                            None,
                            None,
                            Entry::Output(OutputEntry {
                                result: EntryResult::Success(result.clone().into()),
                            }),
                            None,
                        );
                        debug!("Invocation end");
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
                debug!("Invocation done");
            }
        }
    }

    #[tracing::instrument(parent = None, skip(self, waker, message))]
    pub fn handle_user_code_message(
        &mut self,
        entry_name: Option<String>,
        entry_index: Option<u32>,
        message: Entry,
        waker: Option<Waker>,
    ) -> (u32, Option<Bytes>) {
        if self.machine_closed {
            // Return fused
        }
        if let Some(entry_index) = entry_index {
            (entry_index, self.journal.resolve_result(entry_index))
        } else {
            let (entry_index, result) = self.journal.handle_user_code_message(message.clone(), waker);
            if result.is_none() {
                match &message {
                    Entry::Input(_) => {}
                    Entry::Output(output) => {
                        if !self.journal.is_output_replayed() {
                            debug!(
                                "Result does not exist for entry index {:?}, sending output message",
                                entry_index
                            );
                            self.send(
                                PlainRawEntry::new(
                                    PlainEntryHeader::Output,
                                    service_protocol::OutputEntryMessage {
                                        name: entry_name.unwrap_or_default(),
                                        result: match &output.result {
                                            EntryResult::Success(success) => {
                                                Some(service_protocol::output_entry_message::Result::Value(
                                                    success.clone(),
                                                ))
                                            }
                                            EntryResult::Failure(code, message) => {
                                                Some(service_protocol::output_entry_message::Result::Failure(
                                                    Failure {
                                                        code: (*code).into(),
                                                        message: message.clone().to_string(),
                                                    },
                                                ))
                                            }
                                        },
                                    }
                                    .encode_to_vec()
                                    .into(),
                                )
                                .into(),
                            );
                        } else {
                            debug!("Output replayed and matched output message from journal");
                        }
                    }
                    Entry::GetState(get_state) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending get state message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::GetState { is_completed: false },
                                service_protocol::GetStateEntryMessage {
                                    key: get_state.key.clone(),
                                    name: entry_name.unwrap_or_default(),
                                    result: None,
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::SetState(set_state) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending set state message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::SetState,
                                service_protocol::SetStateEntryMessage {
                                    key: set_state.key.clone(),
                                    name: entry_name.unwrap_or_default(),
                                    value: set_state.value.clone(),
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::ClearState(clear_state) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending clear state message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::ClearState,
                                service_protocol::ClearStateEntryMessage {
                                    key: clear_state.key.clone(),
                                    name: entry_name.unwrap_or_default(),
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::GetStateKeys(get_state_keys) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending get state keys message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::GetStateKeys { is_completed: false },
                                service_protocol::GetStateKeysEntryMessage {
                                    name: entry_name.unwrap_or_default(),
                                    result: match get_state_keys.value.as_ref() {
                                        Some(result) => match result {
                                            GetStateKeysResult::Result(keys) => {
                                                Some(get_state_keys_entry_message::Result::Value(
                                                    get_state_keys_entry_message::StateKeys {
                                                        keys: keys.clone(),
                                                    },
                                                ))
                                            }
                                            GetStateKeysResult::Failure(code, message) => {
                                                Some(get_state_keys_entry_message::Result::Failure(Failure {
                                                    code: (*code).into(),
                                                    message: message.clone().to_string(),
                                                }))
                                            }
                                        },
                                        None => None,
                                    },
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::ClearAllState => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending clear all state message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::ClearState,
                                service_protocol::ClearAllStateEntryMessage {
                                    name: entry_name.unwrap_or_default(),
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::GetPromise(get) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending get promise message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::GetPromise { is_completed: false },
                                service_protocol::GetPromiseEntryMessage {
                                    key: get.key.clone().to_string(),
                                    name: entry_name.unwrap_or_default(),
                                    result: None,
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::PeekPromise(peek) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending peek promise message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::PeekPromise { is_completed: false },
                                service_protocol::PeekPromiseEntryMessage {
                                    key: peek.key.clone().to_string(),
                                    name: entry_name.unwrap_or_default(),
                                    result: None,
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::CompletePromise(complete) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending complete promise message",
                            entry_index
                        );
                        let completion = match &complete.completion {
                            EntryResult::Success(value) => {
                                complete_promise_entry_message::Completion::CompletionValue(value.clone())
                            }
                            EntryResult::Failure(code, message) => {
                                complete_promise_entry_message::Completion::CompletionFailure(Failure {
                                    code: (*code).into(),
                                    message: message.to_string(),
                                })
                            }
                        };
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::CompletePromise { is_completed: false },
                                service_protocol::CompletePromiseEntryMessage {
                                    key: complete.key.clone().to_string(),
                                    name: entry_name.unwrap_or_default(),
                                    completion: Some(completion),
                                    result: None,
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::Sleep(sleep) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending sleep message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::Sleep { is_completed: false },
                                service_protocol::SleepEntryMessage {
                                    wake_up_time: sleep.wake_up_time,
                                    name: entry_name.unwrap_or_default(),
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
                                    name: entry_name.unwrap_or_default(),
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
                                    name: entry_name.unwrap_or_default(),
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::Awakeable(awakeable) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending awakeable message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::Awakeable { is_completed: false },
                                service_protocol::AwakeableEntryMessage {
                                    name: entry_name.unwrap_or_default(),
                                    result: None,
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::CompleteAwakeable(complete_awakeable) => {
                        debug!(
                            "Result does not exist for entry index {:?}, sending complete awakeable message",
                            entry_index
                        );
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::CompleteAwakeable {
                                    enrichment_result: (),
                                },
                                service_protocol::CompleteAwakeableEntryMessage {
                                    id: complete_awakeable.id.clone().to_string(),
                                    name: entry_name.unwrap_or_default(),
                                    result: match &complete_awakeable.result {
                                        EntryResult::Success(value) => Some(
                                            complete_awakeable_entry_message::Result::Value(value.clone()),
                                        ),
                                        EntryResult::Failure(code, message) => {
                                            Some(complete_awakeable_entry_message::Result::Failure(Failure {
                                                code: (*code).into(),
                                                message: message.clone().to_string(),
                                            }))
                                        }
                                    },
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::Run(run) => {
                        let result = match &run.result {
                            EntryResult::Success(value) => run_entry_message::Result::Value(value.clone()),
                            EntryResult::Failure(code, message) => {
                                run_entry_message::Result::Failure(Failure {
                                    code: (*code).into(),
                                    message: message.to_string(),
                                })
                            }
                        };
                        self.send(
                            PlainRawEntry::new(
                                PlainEntryHeader::Run,
                                service_protocol::RunEntryMessage {
                                    name: entry_name.unwrap_or_default(),
                                    result: Some(result),
                                }
                                .encode_to_vec()
                                .into(),
                            )
                            .into(),
                        );
                    }
                    Entry::Custom(bytes) => {
                        self.send(
                            PlainRawEntry::new(PlainEntryHeader::Custom { code: 0xfc00 }, bytes.clone())
                                .into(),
                        );
                    }
                }
                (entry_index, None)
            } else {
                debug!(
                    "Result exists for entry index {:?}, result: {:?}",
                    entry_index, result
                );
                (entry_index, result)
            }
        }
    }

    #[tracing::instrument(parent = None, skip(self, waker, message))]
    pub fn write_combinator_order(
        &mut self,
        entry_index: Option<u32>,
        message: Entry,
        waker: Waker,
    ) -> (u32, Option<Bytes>) {
        if let Some(entry_index) = entry_index {
            (entry_index, self.journal.resolve_result(entry_index))
        } else {
            self.journal.increment_user_code_index();
            let entry_index = self.journal.get_user_code_journal_index();
            self.journal.append_entry(message.clone(), waker);
            match &message {
                Entry::Custom(bytes) => {
                    // TODO: Acknowledge flag
                    self.send(
                        PlainRawEntry::new(PlainEntryHeader::Custom { code: 0xfc00 }, bytes.clone()).into(),
                    );
                }
                _ => {}
            }
            (entry_index, None)
        }
    }

    pub fn get_user_code_journal_index(&self) -> u32 {
        return self.journal.get_user_code_journal_index();
    }

    pub fn get_next_user_code_journal_index(&self) -> u32 {
        return self.journal.get_next_user_code_journal_index();
    }

    fn send(&mut self, message: ProtocolMessage) {
        // If in processing or no use calls are performed at all
        if !self.journal.is_replaying() || self.journal.get_user_code_journal_index() == 0 {
            if let Some(ref connection) = self.connection {
                connection.send(message);
            }
        } else {
            debug!(
                "Journal state: {:?}, Skip sending message",
                self.journal.get_state()
            );
        }
    }

    fn hit_suspension(&self) {
        self.suspension_tx.send("suspend".to_string()).unwrap()
    }

    pub fn suspend(&self) {}

    pub fn set_span(&mut self) {
        /*
        println!(
            "{:?} Span flag:{}, Replaying: {}, Next Replaying: {}",
            self.journal.invocation().debug_id,
            self.span_replay_flag,
            self.journal.is_replaying(),
            self.journal.is_next_entry_replaying()
        );
         */
        if self.span_replay_flag == true && self.journal.is_replaying() {
            tracing::Span::current().record("replay", true);
            self.span_replay_flag = false;
        }
        if self.span_replay_flag == false && !self.journal.is_next_entry_replaying() {
            tracing::Span::current().record("replay", false);
            self.span_replay_flag = true;
        }
    }

    pub fn is_replaying(&self) -> bool {
        self.journal.is_replaying()
    }
}

impl RestateStreamConsumer for MutexGuard<'_, StateMachine> {
    fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
        debug!("Machine runtime message handler: {:?}", message);
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
