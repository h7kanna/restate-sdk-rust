use crate::{
    connection::{Connection, RestateStreamConsumer},
    context::RestateContext,
    invocation::Invocation,
    journal::Journal,
    logger::Logger,
    store::LocalStore,
};
use bytes::Bytes;
use parking_lot::Mutex;
use prost::Message;
use restate_sdk_types::{
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

pub(crate) struct StateMachine {
    closed: bool,
    input_channel_closed: bool,
    journal: Journal,
    local_state_store: LocalStore,
    logger: Logger,
    suspension_tx: UnboundedSender<String>,
    connection: Box<dyn Connection>,
}

impl StateMachine {
    pub fn new(connection: Box<dyn Connection>, invocation: Invocation) -> (Self, UnboundedReceiver<String>) {
        let (suspension_tx, suspension_rx) = unbounded_channel();
        (
            Self {
                closed: false,
                input_channel_closed: false,
                journal: Journal::new(invocation),
                local_state_store: LocalStore::new(),
                logger: Logger::new(),
                suspension_tx,
                connection,
            },
            suspension_rx,
        )
    }

    pub async fn invoke(state_machine: Arc<Mutex<StateMachine>>) {
        let ctx = RestateContext::new(state_machine.clone());
        let result = service_fn(ctx, ExecInput {
            test: "hello".to_string(),
        })
        .await;
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
        state_machine
            .lock()
            .send(ProtocolMessage::End(service_protocol::EndMessage {}));
        //state_machine.lock().handle_user_code_message(output.message_type, output.message);
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
        if self.closed {}
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

impl RestateStreamConsumer for &mut StateMachine {
    fn handle(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
        if message.0 == MessageType::Completion {
            if let ProtocolMessage::Completion(message) = message.1 {
                self.journal.handle_runtime_completion_message(message);
            } else {
                // Wrong message type
            }
        } else if message.0 == MessageType::EntryAck {
            if let ProtocolMessage::EntryAck(message) = message.1 {
            } else {
                // Wrong message type
            }
        }

        true
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

async fn service_fn(ctx: RestateContext, name: ExecInput) -> ExecOutput {
    ctx.invoke_service::<String, ExecInput, ExecOutput>("".to_string(), name)
        .await
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_state_machine() {}
}
