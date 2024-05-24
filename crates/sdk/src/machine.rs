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
use restate_sdk_types::{
    protocol,
    protocol::{
        Message::{CompletionMessage, EndMessage, EntryAckMessage, OutputEntryMessage},
        COMPLETION_MESSAGE_TYPE, END_MESSAGE_TYPE, ENTRY_ACK_MESSAGE_TYPE, OUTPUT_ENTRY_MESSAGE_TYPE,
    },
    service_protocol::output_entry_message,
    Message,
};
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
        let output = Message {
            message_type: OUTPUT_ENTRY_MESSAGE_TYPE,
            message: OutputEntryMessage(
                OUTPUT_ENTRY_MESSAGE_TYPE,
                restate_sdk_types::service_protocol::OutputEntryMessage {
                    name: "".to_string(),
                    result: Some(output_entry_message::Result::Value(result.into())),
                },
            ),
            completed: false,
            requires_ack: None,
        };
        println!("{:?} end", output);
        state_machine.lock().send(Message {
            message_type: END_MESSAGE_TYPE,
            message: EndMessage(
                END_MESSAGE_TYPE,
                restate_sdk_types::service_protocol::EndMessage {},
            ),
            completed: false,
            requires_ack: None,
        });
        //state_machine.lock().handle_user_code_message(output.message_type, output.message);
    }

    pub fn handle_runtime_message(&mut self, message: Message) -> Bytes {
        //self.journal.handle_runtime_completion_message();
        Bytes::new()
    }

    pub fn handle_user_code_message(
        &mut self,
        entry_index: u32,
        message: protocol::Message,
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

    fn send(&mut self, message: Message) {
        //self.connection.send(message);
    }

    fn hit_suspension(&self) {
        self.suspension_tx.send("suspend".to_string()).unwrap()
    }

    pub fn suspend(&self) {}
}

impl RestateStreamConsumer for &mut StateMachine {
    fn handle(&mut self, message: Message) -> bool {
        if message.message_type == COMPLETION_MESSAGE_TYPE {
            if let CompletionMessage(_, message) = message.message {
                self.journal.handle_runtime_completion_message(message);
            } else {
                // Wrong message type
            }
        } else if message.message_type == ENTRY_ACK_MESSAGE_TYPE {
            if let EntryAckMessage(_, message) = message.message {
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
