use crate::{connection::RestateStreamConsumer, store::LocalStore};
use bytes::Bytes;
use dashmap::DashMap;
use restate_sdk_types::{
    journal::{Entry, InputEntry},
    service_protocol::StartMessage,
};
use restate_service_protocol::{
    codec::ProtobufRawEntryCodec,
    message::{MessageType, ProtocolMessage},
};
use std::{cmp::PartialEq, collections::HashMap};

#[derive(Copy, Clone, PartialEq)]
enum State {
    ExpectingStart,
    ExpectingInput,
    ExpectingFurtherReplay,
    Complete,
}

pub(crate) struct Invocation {
    pub id: Bytes,
    pub debug_id: Option<String>,
    pub number_entries_to_replay: u32,
    pub replay_entries: DashMap<u32, Entry>,
    pub invocation_value: Option<Bytes>,
    pub invocation_headers: Option<HashMap<String, String>>,
    pub local_state_store: Option<LocalStore>,
    pub user_key: Option<String>,
}

pub(crate) struct InvocationBuilder {
    state: State,
    replay_index: u32,
    replay_entries: DashMap<u32, Entry>,
    id: Option<Bytes>,
    debug_id: Option<String>,
    known_entries: u32,
    invocation_value: Option<Bytes>,
    invocation_headers: Option<HashMap<String, String>>,
    local_state_store: Option<LocalStore>,
    user_key: Option<String>,
}

impl InvocationBuilder {
    pub fn new() -> Self {
        Self {
            state: State::ExpectingStart,
            replay_index: 0,
            replay_entries: DashMap::new(),
            id: None,
            debug_id: None,
            known_entries: 0,
            invocation_value: None,
            invocation_headers: None,
            local_state_store: None,
            user_key: None,
        }
    }

    pub fn build(self) -> Invocation {
        if !self.is_complete() {
            // Error
        }
        Invocation {
            id: self.id.unwrap(),
            debug_id: self.debug_id,
            number_entries_to_replay: self.known_entries,
            replay_entries: self.replay_entries,
            invocation_value: self.invocation_value,
            invocation_headers: self.invocation_headers,
            local_state_store: self.local_state_store,
            user_key: self.user_key,
        }
    }

    fn handle_start_message(&mut self, message: StartMessage) {
        self.known_entries = message.known_entries;
        self.id = Some(message.id);
        self.debug_id = Some(message.debug_id);
        self.user_key = Some(message.key);
        self.local_state_store = Some(LocalStore::new(message.partial_state, message.state_map))
    }

    fn handle_input_message(&mut self, message: InputEntry) {
        self.invocation_value = Some(message.value);
    }

    fn deserialize_entry(&mut self, message: ProtocolMessage) -> Option<Entry> {
        if let ProtocolMessage::UnparsedEntry(raw_entry) = message {
            let expected_entry = raw_entry.deserialize_entry_ref::<ProtobufRawEntryCodec>();
            println!("Entry received {:?}", expected_entry);
            //TODO: Handle error
            Some(expected_entry.unwrap())
        } else {
            None
        }
    }

    fn append_replay_entry(&mut self, entry: Entry) {
        self.replay_entries.insert(self.replay_index, entry);
        self.replay_index += 1;
    }

    fn is_complete(&self) -> bool {
        self.state == State::Complete
    }

    fn check_state(
        &self,
        state: State,
        expected: MessageType,
        actual: MessageType,
        message: &ProtocolMessage,
    ) {
    }
}

impl RestateStreamConsumer for InvocationBuilder {
    fn handle_message(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
        match self.state {
            State::ExpectingStart => {
                self.check_state(self.state, MessageType::Start, message.0, &message.1);
                match message.1 {
                    ProtocolMessage::Start(start_message) => {
                        self.handle_start_message(start_message);
                    }
                    _ => {
                        // Invalid
                    }
                }
                self.state = State::ExpectingInput;
                return false;
            }
            State::ExpectingInput => {
                self.check_state(self.state, MessageType::InputEntry, message.0, &message.1);
                let entry = self.deserialize_entry(message.1).unwrap();
                if let Entry::Input(input) = &entry {
                    self.handle_input_message(input.clone());
                }
                self.append_replay_entry(entry);
            }
            State::ExpectingFurtherReplay => {
                let entry = self.deserialize_entry(message.1).unwrap();
                self.append_replay_entry(entry);
            }
            State::Complete => {
                // Error
            }
        }
        if self.replay_entries.len() == self.known_entries as usize {
            self.state = State::Complete;
        } else {
            self.state = State::ExpectingFurtherReplay;
        }
        self.is_complete()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_invocation() {}
}
