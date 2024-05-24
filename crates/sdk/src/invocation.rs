use crate::{connection::RestateStreamConsumer, store::LocalStore};
use bytes::Bytes;
use dashmap::DashMap;
use restate_sdk_types::{
    journal::{Entry, InputEntry},
    service_protocol::StartMessage,
};
use restate_service_protocol::{
    codec::ProtobufRawEntryCodec,
    message::{MessageHeader, MessageType, ProtocolMessage},
};
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
    pub replay_entries: DashMap<u32, Entry>,
    pub invocation_value: Option<Bytes>,
}

pub(crate) struct InvocationBuilder {
    state: State,
    runtime_replay_index: u32,
    replay_entries: DashMap<u32, Entry>,
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
        self.replay_entries.insert(self.runtime_replay_index, entry);
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
            invocation_value: self.invocation_value,
        }
    }

    fn check_state(&self, state: State, expected: MessageType, message: &ProtocolMessage) {}
}

impl RestateStreamConsumer for &mut InvocationBuilder {
    fn handle(&mut self, message: (MessageType, ProtocolMessage)) -> bool {
        match self.state {
            State::ExpectingStart => {
                self.check_state(self.state, message.0, &message.1);
                match message.1 {
                    ProtocolMessage::Start(start_message) => {
                        self.handle_start_message(start_message);
                    }
                    _ => {
                        // Invalid
                    }
                }
            }
            State::ExpectingInput => {
                self.check_state(self.state, message.0, &message.1);
                let entry = self.deserialize_entry(message.1).unwrap();
                match &entry {
                    Entry::Input(input) => {
                        self.handle_input_message(input.clone());
                    }
                    _ => {}
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
