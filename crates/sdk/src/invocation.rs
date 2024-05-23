use crate::{connection::RestateStreamConsumer, store::LocalStore};
use bytes::Bytes;
use dashmap::DashMap;
use restate_sdk_types::service_protocol::{InputEntryMessage, StartMessage};
use restate_sdk_types::{protocol, Message};
use std::cmp::PartialEq;

#[derive(Copy, Clone, PartialEq)]
enum State {
    ExpectingStart = 0,
    ExpectingInput = 1,
    ExpectingFurtherReplay = 2,
    Complete = 3,
}

pub(crate) struct Invocation {
    pub nb_entries_to_replay: u32,
    pub replay_entries: DashMap<u32, Message>,
}

pub(crate) struct InvocationBuilder {
    state: State,
    runtime_replay_index: u32,
    replay_entries: DashMap<u32, Message>,
    id: Option<Bytes>,
    debug_id: Option<String>,
    nb_entries_to_replay: u32,
    invocation_value: Option<Bytes>,
    invocation_headers: Option<Bytes>,
    local_state_store: LocalStore,
    user_key: Option<String>,
}

impl InvocationBuilder {
    pub fn new() -> Self {
        Self {
            state: State::ExpectingStart,
            runtime_replay_index: 0,
            replay_entries: DashMap::new(),
            id: None,
            debug_id: None,
            nb_entries_to_replay: 0,
            invocation_value: None,
            invocation_headers: None,
            local_state_store: LocalStore::new(),
            user_key: None,
        }
    }

    fn handle_start_message(&mut self, message: StartMessage) {
        self.nb_entries_to_replay = message.known_entries;
        self.id = Some(message.id);
        self.debug_id = Some(message.debug_id);
        self.user_key = Some(message.key)
    }

    fn handle_input_message(&mut self, message: InputEntryMessage) {
        self.invocation_value = Some(message.value);
    }

    fn append_replay_entry(&mut self, message: Message) {
        self.replay_entries.insert(self.runtime_replay_index, message);
        self.runtime_replay_index += 1;
    }

    fn is_complete(&self) -> bool {
        self.state == State::Complete
    }

    pub fn build(self) -> Invocation {
        if !self.is_complete() {
            // Error
        }
        Invocation {
            nb_entries_to_replay: self.nb_entries_to_replay,
            replay_entries: self.replay_entries,
        }
    }

    fn check_state(&self, state: State, expected: u16, message: &Message) {}
}

impl RestateStreamConsumer for &mut InvocationBuilder {
    async fn handle(&mut self, message: Message) -> bool {
        match self.state {
            State::ExpectingStart => {
                self.check_state(self.state, message.message_type, &message);
                match message.message {
                    protocol::Message::StartMessage(_, start_message) => {
                        self.handle_start_message(start_message);
                    }
                    _ => {
                        // Invalid
                    }
                }
            }
            State::ExpectingInput => {
                self.check_state(self.state, message.message_type, &message);
                match &message.message {
                    protocol::Message::InputEntryMessage(_, input_message) => {
                        self.handle_input_message(input_message.clone());
                    }
                    _ => {
                        // Invalid
                    }
                }
                self.append_replay_entry(message);
            }
            State::ExpectingFurtherReplay => {
                self.append_replay_entry(message);
            }
            State::Complete => {
                // Error
            }
        }
        if self.replay_entries.len() == self.nb_entries_to_replay as usize {
            self.state = State::Complete;
        } else {
            self.state = State::ExpectingFurtherReplay;
        }
        self.state == State::Complete
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_invocation() {}
}
