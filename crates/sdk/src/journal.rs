use crate::invocation::Invocation;
use bytes::Bytes;
use dashmap::DashMap;
use futures_util::task::waker;
use restate_sdk_types::{
    journal::{Entry, EntryResult, InputEntry},
    service_protocol::{
        call_entry_message, completion_message, CompletionMessage, EntryAckMessage, InputEntryMessage,
    },
};
use std::{cmp::PartialEq, task::Waker};
use tracing::info;

#[derive(PartialEq)]
pub enum NewExecutionState {
    REPLAYING,
    PROCESSING,
    CLOSED,
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub entry: Entry,
    pub waker: Option<Waker>,
}

pub struct Journal {
    state: NewExecutionState,
    user_code_journal_index: u32,
    pending_entries: DashMap<u32, JournalEntry>,
    invocation: Invocation,
}

impl Journal {
    pub fn new(invocation: Invocation) -> Self {
        let mut journal = Self {
            state: NewExecutionState::REPLAYING,
            user_code_journal_index: 0,
            pending_entries: Default::default(),
            invocation,
        };

        if journal.invocation.replay_entries.contains_key(&0) {
            // The First message of replay entries needs to be InputStreamMessage
        } else {
            let input_message = journal.invocation.replay_entries.get(&0).unwrap().clone();
            if let Entry::Input(input) = input_message {
                journal.handle_input_message(input);
            } else {
                // Error should be input message
            }
        }
        journal
    }

    fn handle_input_message(&mut self, input: InputEntry) {
        if self.invocation.number_entries_to_replay == 1 {
            self.transition_state(NewExecutionState::PROCESSING);
        }
        self.pending_entries.insert(0, JournalEntry {
            entry: Entry::Input(input),
            waker: None,
        });
    }

    fn transition_state(&mut self, new_state: NewExecutionState) {
        match self.state {
            NewExecutionState::CLOSED => {
                // If the state is already closed, then the state cannot transition anymore
                if self.state == new_state {
                    self.state = new_state;
                }
            }
            _ => {
                self.state = new_state;
            }
        }
    }

    fn increment_user_code_index(&mut self) {
        self.user_code_journal_index += 1;
        if self.user_code_journal_index == self.invocation.number_entries_to_replay
            && self.state == NewExecutionState::REPLAYING
        {
            self.transition_state(NewExecutionState::PROCESSING)
        }
    }

    pub fn handle_user_code_message(
        &mut self,
        entry_index: u32,
        message: Entry,
        waker: Waker,
    ) -> Option<Bytes> {
        if entry_index != self.get_user_code_journal_index() {
            self.increment_user_code_index();
            match self.state {
                NewExecutionState::REPLAYING => {
                    if let Some(replay_entry) =
                        self.invocation.replay_entries.get(&self.user_code_journal_index)
                    {
                        let journal_entry = JournalEntry {
                            entry: message,
                            waker: Some(waker),
                        };
                        let replay_message = replay_entry.clone();
                        return self.handle_replay(entry_index, replay_message, journal_entry);
                    } else {
                        // Illegal
                    }
                }
                NewExecutionState::PROCESSING => self.handle_processing(entry_index, message, waker),
                NewExecutionState::CLOSED => {}
            }
        }
        None
    }

    fn handle_replay(&self, entry_index: u32, replay_entry: Entry, entry: JournalEntry) -> Option<Bytes> {
        match replay_entry {
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
            Entry::Sleep(_) => {}
            Entry::Call(call) => {
                if let Some(result) = call.result {
                    match result {
                        EntryResult::Success(value) => return Some(value),
                        EntryResult::Failure(code, value) => {}
                    }
                }
            }
            Entry::OneWayCall(_) => {}
            Entry::Awakeable(_) => {}
            Entry::CompleteAwakeable(_) => {}
            Entry::Run(_) => {}
            Entry::Custom(_) => {}
        }
        None
    }

    fn handle_processing(&self, entry_index: u32, message: Entry, waker: Waker) {
        match message {
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
            Entry::Sleep(_) => {}
            entry @ Entry::Call(_) => {
                self.append_entry(entry, waker);
            }
            Entry::OneWayCall(_) => {}
            Entry::Awakeable(_) => {}
            Entry::CompleteAwakeable(_) => {}
            Entry::Run(_) => {}
            Entry::Custom(_) => {}
        }
    }

    fn resolve_result(&self) {}

    pub fn handle_runtime_completion_message(&self, message: CompletionMessage) {
        let journal_entry = self.pending_entries.get_mut(&message.entry_index);
        if let Some(mut journal_entry) = journal_entry {
            match message.result {
                Some(result) => match result {
                    completion_message::Result::Empty(_) => {}
                    completion_message::Result::Value(value) => {
                        info!("{:?}", value);
                    }
                    completion_message::Result::Failure(_) => {}
                },
                None => {}
            }
            if let Some(mut waker) = journal_entry.waker.take() {
                waker.wake();
            }
        } else {
            return;
        }
    }

    pub fn handle_runtime_entry_ack_message(&self, message: EntryAckMessage) {
        if let Some((_, entry)) = self.pending_entries.remove(&message.entry_index) {
            if let Some(waker) = entry.waker {
                waker.wake();
            }
        }
    }

    pub fn append_entry(&self, message: Entry, waker: Waker) {
        self.pending_entries
            .insert(self.user_code_journal_index, JournalEntry {
                entry: message,
                waker: Some(waker),
            });
    }

    pub fn is_unresolved(&self, index: u32) -> bool {
        self.pending_entries.get(&index).is_some()
    }

    pub fn is_closed(&self) -> bool {
        return self.state == NewExecutionState::CLOSED;
    }

    pub fn is_processing(&self) -> bool {
        return self.state == NewExecutionState::PROCESSING;
    }

    pub fn is_replaying(&self) -> bool {
        return self.state == NewExecutionState::REPLAYING;
    }

    pub fn get_user_code_journal_index(&self) -> u32 {
        return self.user_code_journal_index;
    }

    pub fn get_next_user_code_journal_index(&self) -> u32 {
        return self.user_code_journal_index + 1;
    }

    pub fn close(&mut self) {
        self.transition_state(NewExecutionState::CLOSED);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_journal() {}
}
