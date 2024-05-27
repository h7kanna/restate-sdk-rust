use crate::{
    connection::{MessageSender, RestateStreamConsumer},
    context::RestateContext,
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
        Entry,
    },
    service_protocol,
};
use restate_service_protocol::message::{MessageType, ProtocolMessage};
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc, task::Waker};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const SUSPENSION_MILLIS: u32 = 30000;

pub(crate) struct StateMachine {
    journal: Journal,
    machine_closed: bool,
    input_channel_closed: bool,
    local_state_store: Option<LocalStore>,
    logger: Logger,
    suspension_tx: UnboundedSender<String>,
    connection: Box<dyn MessageSender>,
    protocol_mode: ProtocolMode,
    input: Option<Bytes>,
}

impl StateMachine {
    pub fn new(
        connection: Box<dyn MessageSender>,
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
                protocol_mode: ProtocolMode::BidiStream,
                input,
            },
            suspension_rx,
        )
    }

    pub async fn invoke<F, I, R>(handler: F, state_machine: Arc<Mutex<StateMachine>>)
    where
        for<'a> I: Serialize + Deserialize<'a>,
        for<'a> R: Serialize + Deserialize<'a>,
        F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
    {
        let input = state_machine.lock().input.clone().unwrap();
        let input = serde_json::from_slice(&input.to_vec()).unwrap();
        let ctx = RestateContext::new(state_machine.clone());
        match handler(ctx, input).await {
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
                println!("{:?} end", output);
            }
            Err(err) => {}
        };
        state_machine
            .lock()
            .send(ProtocolMessage::End(service_protocol::EndMessage {}));
    }

    pub fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
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

    pub fn handle_runtime_message(&mut self, message: ProtocolMessage) -> Bytes {
        match message {
            ProtocolMessage::Completion(completion) => {
                self.journal.handle_runtime_completion_message(completion);
            }
            _ => {
                // Error
            }
        }
        Bytes::new()
    }

    pub fn handle_user_code_message(
        &mut self,
        entry_index: u32,
        message: Entry,
        waker: Waker,
    ) -> Option<Bytes> {
        if self.machine_closed {}
        let result = self
            .journal
            .handle_user_code_message(entry_index, message.clone(), waker);
        if result.is_none() {
            //self.send(message);
            None
        } else {
            println!("There  index {:?}, result: {:?}", entry_index, result);
            result
        }
    }

    pub fn get_next_user_code_journal_index(&self) -> u32 {
        return self.journal.get_next_user_code_journal_index();
    }

    fn send(&mut self, message: ProtocolMessage) {
        self.connection.send(message);
    }

    fn hit_suspension(&self) {
        self.suspension_tx.send("suspend".to_string()).unwrap()
    }

    pub fn suspend(&self) {}
}


impl RestateStreamConsumer for MutexGuard<'_, StateMachine> {
    fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
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
