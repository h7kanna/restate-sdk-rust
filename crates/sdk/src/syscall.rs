use crate::machine::StateMachine;
use bytes::Bytes;
use parking_lot::Mutex;
use prost::Message;
use restate_sdk_types::journal::{
    AwakeableEntry, CompletePromiseEntry, Entry, GetPromiseEntry, InvokeEntry, RunEntry, SleepEntry,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

macro_rules! future_impl {
    ($future:tt, $entry:tt) => {
        impl $future {
            pub fn new(entry: $entry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
                let entry_index = state_machine.lock().get_next_user_code_journal_index();
                Self {
                    entry_index,
                    entry,
                    state_machine,
                }
            }

            pub fn entry(&self) -> u32 {
                self.entry_index
            }
        }
    };
}

pub struct AwakeableFuture {
    entry_index: u32,
    entry: AwakeableEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(AwakeableFuture, AwakeableEntry);

impl Future for AwakeableFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Awakeable(self.entry.clone()),
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
    entry: SleepEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(SleepFuture, SleepEntry);

impl Future for SleepFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Sleep(self.entry.clone()),
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
    entry: RunEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(RunFuture, RunEntry);

impl Future for RunFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::Run(self.entry.clone()),
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

pub struct GetPromiseFuture {
    entry_index: u32,
    entry: GetPromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(GetPromiseFuture, GetPromiseEntry);

impl Future for GetPromiseFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::GetPromise(self.entry.clone()),
            cx.waker().clone(),
        ) {
            println!("GetPromise Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("GetPromise Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_call_service() {}
}
