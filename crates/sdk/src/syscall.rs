use crate::machine::StateMachine;
use bytes::Bytes;
use parking_lot::Mutex;
use prost::Message;
use restate_sdk_types::journal::{AwakeableEntry, Entry, InvokeEntry, RunEntry, SleepEntry};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct AwakeableFuture {
    entry_index: u32,
    awakeable_entry: AwakeableEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl AwakeableFuture {
    pub fn new(awakeable_entry: AwakeableEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            awakeable_entry,
            state_machine,
        }
    }

    pub fn entry(&self) -> u32 {
        self.entry_index
    }
}

impl Future for AwakeableFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Awakeable(self.awakeable_entry.clone()),
            cx.waker().clone(),
        ) {
            println!("Run Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("Run Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}


pub struct SleepFuture {
    entry_index: u32,
    sleep_entry: SleepEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl SleepFuture {
    pub fn new(sleep_entry: SleepEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            sleep_entry,
            state_machine,
        }
    }
}

impl Future for SleepFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Sleep(self.sleep_entry.clone()),
            cx.waker().clone(),
        ) {
            println!("Sleep Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("Sleep Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct RunFuture {
    entry_index: u32,
    run_entry: RunEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl RunFuture {
    pub fn new(run_entry: RunEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            run_entry,
            state_machine,
        }
    }
}

impl Future for RunFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Run(self.run_entry.clone()),
            cx.waker().clone(),
        ) {
            println!("Run Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("Run Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct CallServiceFuture<T> {
    entry_index: u32,
    invoke_entry: InvokeEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    _ret: PhantomData<T>,
}

impl<T> CallServiceFuture<T> {
    pub fn new(invoke_entry: InvokeEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            invoke_entry,
            state_machine,
            _ret: PhantomData,
        }
    }
}

impl<T> Future for CallServiceFuture<T> {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Call(self.invoke_entry.clone()),
            cx.waker().clone(),
        ) {
            println!("Call Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("Call Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_call_service() {}
}
