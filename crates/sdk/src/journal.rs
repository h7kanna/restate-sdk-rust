use crate::invocation::Invocation;
use bytes::Bytes;
use dashmap::DashMap;
use futures_util::task::waker;
use prost::Message;
use restate_sdk_types::{
    journal::{
        CompleteResult, CompletionResult, Entry, EntryResult, GetStateKeysResult, InputEntry, RunEntry,
        SleepResult,
    },
    service_protocol::{
        awakeable_entry_message, call_entry_message, complete_promise_entry_message, completion_message,
        get_state_keys_entry_message, CompletionMessage, EntryAckMessage, InputEntryMessage,
    },
};
use std::{cmp::PartialEq, task::Waker};
use tracing::info;

#[derive(Debug, PartialEq)]
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
        info!(
            "Journal state: {:?}, Entries to replay: {}",
            journal.state, journal.invocation.number_entries_to_replay
        );
        journal
    }

    pub fn invocation(&self) -> &Invocation {
        &self.invocation
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
        info!(
            "Journal state: {:?}, Journal user index: {}",
            self.state, self.user_code_journal_index
        );
    }

    fn increment_user_code_index(&mut self) {
        self.user_code_journal_index += 1;
        if self.user_code_journal_index == self.invocation.number_entries_to_replay
            && self.state == NewExecutionState::REPLAYING
        {
            self.transition_state(NewExecutionState::PROCESSING)
        }
    }

    #[tracing::instrument(skip(self, entry, waker))]
    pub fn handle_user_code_message(
        &mut self,
        entry_index: u32,
        entry: Entry,
        waker: Option<Waker>,
    ) -> Option<Bytes> {
        info!(
            "Handle user code entry_index: {}, journal_index: {}, state: {:?}, replay_entries: {}",
            entry_index,
            self.get_user_code_journal_index(),
            self.state,
            self.invocation.number_entries_to_replay
        );
        if entry_index != self.get_user_code_journal_index() {
            self.increment_user_code_index();
            match self.state {
                NewExecutionState::REPLAYING => {
                    if let Some((_, replay_entry)) = self
                        .invocation
                        .replay_entries
                        .remove(&self.user_code_journal_index)
                    {
                        let journal_entry = JournalEntry { entry, waker };
                        return self.handle_replay(entry_index, replay_entry, journal_entry);
                    } else {
                        // Illegal
                    }
                }
                NewExecutionState::PROCESSING => self.handle_processing(entry_index, entry, waker),
                NewExecutionState::CLOSED => {}
            }
        } else {
            if let Some((_, pending)) = self.pending_entries.remove(&entry_index) {
                match &pending.entry {
                    Entry::Input(_) => {}
                    Entry::Output(_) => {}
                    Entry::GetState(get_state) => {
                        if let Some(result) = get_state.value.as_ref() {
                            return match result {
                                CompletionResult::Empty => Some(Bytes::new()),
                                CompletionResult::Success(bytes) => Some(bytes.clone()),
                                CompletionResult::Failure(_, _) => None,
                            };
                        }
                    }
                    Entry::SetState(_) => return Some(Bytes::new()),
                    Entry::ClearState(_) => return Some(Bytes::new()),
                    Entry::GetStateKeys(get_state_keys) => {
                        if let Some(result) = get_state_keys.value.as_ref() {
                            return match result {
                                GetStateKeysResult::Result(value) => Some(Bytes::new()),
                                GetStateKeysResult::Failure(_, _) => Some(Bytes::new()),
                            };
                        }
                    }
                    Entry::ClearAllState => return Some(Bytes::new()),
                    Entry::GetPromise(get) => {
                        if let Some(result) = get.value.as_ref() {
                            match result {
                                EntryResult::Success(success) => {
                                    return Some(success.clone());
                                }
                                EntryResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::PeekPromise(peek) => {
                        if let Some(result) = peek.value.as_ref() {
                            match result {
                                CompletionResult::Empty => {
                                    return Some(Bytes::new());
                                }
                                CompletionResult::Success(success) => {
                                    return Some(success.clone());
                                }
                                CompletionResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::CompletePromise(complete) => {
                        if let Some(result) = complete.value.as_ref() {
                            match result {
                                CompleteResult::Done => {
                                    return Some(Bytes::new());
                                }
                                CompleteResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::Sleep(sleep) => {
                        if let Some(result) = sleep.result.as_ref() {
                            match result {
                                SleepResult::Fired => {
                                    info!("Sleep fired for entry index: {}", entry_index);
                                    return Some(Bytes::new());
                                }
                                SleepResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::Call(call) => {
                        if let Some(result) = call.result.as_ref() {
                            match result {
                                EntryResult::Success(success) => {
                                    return Some(success.clone());
                                }
                                EntryResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::OneWayCall(_) => {}
                    Entry::Awakeable(awakeaable) => {
                        if let Some(result) = awakeaable.result.as_ref() {
                            match result {
                                EntryResult::Success(success) => {
                                    return Some(success.clone());
                                }
                                EntryResult::Failure(_, _) => {}
                            }
                        }
                    }
                    Entry::CompleteAwakeable(_) => {}
                    Entry::Run(run) => match &run.result {
                        EntryResult::Success(value) => {
                            return Some(value.clone());
                        }
                        EntryResult::Failure(_, _) => {}
                    },
                    Entry::Custom(_) => {}
                }
            }
        }
        None
    }

    fn handle_replay(&mut self, entry_index: u32, replay_entry: Entry, entry: JournalEntry) -> Option<Bytes> {
        match replay_entry {
            Entry::Input(_) => {}
            Entry::Output(output) => {
                self.handle_output_message(entry_index);
                match output.result {
                    EntryResult::Success(success) => {
                        return Some(success);
                    }
                    EntryResult::Failure(_, _) => {}
                }
            }
            Entry::GetState(get_state) => {
                if let Some(result) = get_state.value {
                    return match result {
                        CompletionResult::Empty => Some(Bytes::new()),
                        CompletionResult::Success(bytes) => Some(bytes),
                        CompletionResult::Failure(_, _) => None,
                    };
                }
            }
            Entry::SetState(_) => return Some(Bytes::new()),
            Entry::ClearState(_) => return Some(Bytes::new()),
            Entry::GetStateKeys(get_state_keys) => {
                if let Some(result) = get_state_keys.value {
                    return match result {
                        GetStateKeysResult::Result(value) => Some(Bytes::new()),
                        GetStateKeysResult::Failure(_, _) => Some(Bytes::new()),
                    };
                }
            }
            Entry::ClearAllState => return Some(Bytes::new()),
            Entry::GetPromise(get_promise) => {
                if let Some(result) = get_promise.value {
                    match result {
                        EntryResult::Success(value) => return Some(value),
                        EntryResult::Failure(code, value) => {}
                    }
                }
            }
            Entry::PeekPromise(peek_promise) => {
                if let Some(result) = peek_promise.value {
                    match result {
                        CompletionResult::Empty => {
                            return Some(Bytes::new());
                        }
                        CompletionResult::Success(value) => {
                            return Some(value);
                        }
                        CompletionResult::Failure(_, _) => {}
                    }
                }
            }
            Entry::CompletePromise(complete_promise) => match complete_promise.completion {
                EntryResult::Success(value) => return Some(value),
                EntryResult::Failure(code, value) => {}
            },
            Entry::Sleep(sleep) => {
                if let Some(result) = sleep.result.as_ref() {
                    match result {
                        SleepResult::Fired => {
                            return Some(Bytes::new());
                        }
                        SleepResult::Failure(_, _) => {}
                    }
                }
            }
            Entry::Call(call) => {
                if let Some(result) = call.result {
                    match result {
                        EntryResult::Success(value) => return Some(value),
                        EntryResult::Failure(code, value) => {}
                    }
                }
            }
            Entry::OneWayCall(_) => {}
            Entry::Awakeable(awakeable) => {
                if let Some(result) = awakeable.result {
                    match result {
                        EntryResult::Success(value) => return Some(value),
                        EntryResult::Failure(code, value) => {}
                    }
                }
            }
            Entry::CompleteAwakeable(_) => {}
            Entry::Run(run) => match run.result {
                EntryResult::Success(value) => {
                    return Some(value.clone());
                }
                EntryResult::Failure(_, _) => {}
            },
            Entry::Custom(_) => {}
        }
        None
    }

    fn handle_processing(&mut self, entry_index: u32, entry: Entry, waker: Option<Waker>) {
        match entry {
            Entry::Input(_) => {}
            Entry::Output(_) => {
                self.handle_output_message(entry_index);
            }
            Entry::GetState(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::SetState(_) => {
                //self.append_entry(entry, waker);
            }
            Entry::ClearState(_) => {
                //self.append_entry(entry, waker);
            }
            Entry::GetStateKeys(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::ClearAllState => {
                //self.append_entry(entry, waker.unwrap());
            }
            Entry::GetPromise(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::PeekPromise(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::CompletePromise(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::Sleep(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::Call(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::OneWayCall(_) => {}
            Entry::Awakeable(_) => {
                self.append_entry(entry, waker.unwrap());
            }
            Entry::CompleteAwakeable(_) => {}
            Entry::Run(_) => {}
            Entry::Custom(_) => {}
        }
    }

    fn resolve_result(&self) {}

    #[tracing::instrument(parent = None, skip(self, message))]
    pub fn handle_runtime_completion_message(&self, message: CompletionMessage) {
        let journal_entry = self.pending_entries.get_mut(&message.entry_index);
        if let Some(mut journal_entry) = journal_entry {
            info!("Journal runtime message entry: {:?}", journal_entry);
            match &mut journal_entry.entry {
                Entry::GetState(get_state) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {
                            get_state.value = Some(CompletionResult::Empty)
                        }
                        completion_message::Result::Value(ref value) => {
                            info!("{:?}", value);
                            info!("Journal runtime message get state value: {:?}", result);
                            get_state.value = Some(CompletionResult::Success(value.clone()));
                        }
                        completion_message::Result::Failure(failure) => {
                            get_state.value = Some(CompletionResult::Failure(
                                failure.code.into(),
                                failure.message.into(),
                            ));
                        }
                    },
                    None => {}
                },
                Entry::GetStateKeys(get_state_keys) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {
                            get_state_keys.value = Some(GetStateKeysResult::Result(vec![]));
                        }
                        completion_message::Result::Value(value) => {
                            let result = get_state_keys_entry_message::StateKeys::decode(value).unwrap();
                            get_state_keys.value = Some(GetStateKeysResult::Result(result.keys));
                        }
                        completion_message::Result::Failure(failure) => {
                            get_state_keys.value = Some(GetStateKeysResult::Failure(
                                failure.code.into(),
                                failure.message.into(),
                            ));
                        }
                    },
                    None => {}
                },
                Entry::GetPromise(get_promise) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {}
                        completion_message::Result::Value(ref value) => {
                            info!("{:?}", value);
                            info!("Journal runtime message get promise value: {:?}", result);
                            get_promise.value = Some(EntryResult::Success(value.clone()));
                        }
                        completion_message::Result::Failure(failure) => {
                            get_promise.value =
                                Some(EntryResult::Failure(failure.code.into(), failure.message.into()));
                        }
                    },
                    None => {}
                },
                Entry::PeekPromise(peek_promise) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {
                            peek_promise.value = Some(CompletionResult::Empty);
                        }
                        completion_message::Result::Value(ref value) => {
                            info!("{:?}", value);
                            info!("Journal runtime message get promise value: {:?}", result);
                            peek_promise.value = Some(CompletionResult::Success(value.clone()));
                        }
                        completion_message::Result::Failure(failure) => {
                            peek_promise.value = Some(CompletionResult::Failure(
                                failure.code.into(),
                                failure.message.into(),
                            ));
                        }
                    },
                    None => {}
                },
                Entry::CompletePromise(complete_promise) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) | completion_message::Result::Value(_) => {
                            info!("Journal runtime message get promise value: {:?}", result);
                            complete_promise.value = Some(CompleteResult::Done);
                        }
                        completion_message::Result::Failure(failure) => {
                            complete_promise.value = Some(CompleteResult::Failure(
                                failure.code.into(),
                                failure.message.into(),
                            ));
                        }
                    },
                    None => {}
                },
                Entry::Sleep(sleep) => {
                    match message.result {
                        Some(result) => match result {
                            completion_message::Result::Empty(_) | completion_message::Result::Value(_) => {
                                info!("Journal runtime message sleep value: {:?}", result);
                                sleep.result = Some(SleepResult::Fired);
                            }
                            completion_message::Result::Failure(_) => {}
                        },
                        None => {}
                    }
                    //sleep.result = Some(SleepResult::Fired);
                }
                Entry::Call(call) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {}
                        completion_message::Result::Value(value) => {
                            info!("{:?}", value);
                            info!("Journal runtime message call value: {:?}", value);
                            call.result = Some(EntryResult::Success(value));
                        }
                        completion_message::Result::Failure(failure) => {
                            call.result =
                                Some(EntryResult::Failure(failure.code.into(), failure.message.into()));
                        }
                    },
                    None => {}
                },
                Entry::Awakeable(awakeable) => match message.result {
                    Some(result) => match result {
                        completion_message::Result::Empty(_) => {}
                        completion_message::Result::Value(value) => {
                            info!("{:?}", value);
                            info!("Journal runtime message awakeable value: {:?}", value);
                            awakeable.result = Some(EntryResult::Success(value));
                        }
                        completion_message::Result::Failure(failure) => {
                            awakeable.result =
                                Some(EntryResult::Failure(failure.code.into(), failure.message.into()));
                        }
                    },
                    None => {}
                },
                _ => {}
            }
            if let Some(mut waker) = journal_entry.waker.take() {
                info!("Journal runtime waking up: {:?}", journal_entry.entry);
                waker.wake();
            }
        } else {
            return;
        }
    }

    #[tracing::instrument(parent = None, skip(self, message))]
    pub fn handle_runtime_entry_ack_message(&self, message: EntryAckMessage) {
        if let Some((_, entry)) = self.pending_entries.remove(&message.entry_index) {
            if let Some(waker) = entry.waker {
                waker.wake();
            }
        }
    }

    fn handle_output_message(&mut self, entry_index: u32) {
        self.transition_state(NewExecutionState::CLOSED);
        self.pending_entries.remove(&0);
    }

    pub fn append_entry(&self, entry: Entry, waker: Waker) {
        self.pending_entries
            .insert(self.user_code_journal_index, JournalEntry {
                entry,
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
