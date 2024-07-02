use crate::machine::StateMachine;
use bytes::Bytes;
use parking_lot::Mutex;
use prost::Message;
use restate_sdk_types::{
    journal::{
        AwakeableEntry, ClearStateEntry, CompletePromiseEntry, Entry, GetPromiseEntry, GetStateEntry,
        GetStateKeysEntry, InvokeEntry, PeekPromiseEntry, RunEntry, SetStateEntry, SleepEntry,
    },
    service_protocol::get_state_keys_entry_message,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

macro_rules! future_impl {
    ($future:ident, $entry:ident) => {
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

pub struct GetStateFuture {
    entry_index: u32,
    entry: GetStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(GetStateFuture, GetStateEntry);

impl Future for GetStateFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::GetState(self.entry.clone()),
            Some(cx.waker().clone()),
        ) {
            println!("GetState Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("GetState Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct GetStateKeysFuture {
    entry_index: u32,
    entry: GetStateKeysEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(GetStateKeysFuture, GetStateKeysEntry);

impl Future for GetStateKeysFuture {
    type Output = Vec<Bytes>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::GetStateKeys(self.entry.clone()),
            Some(cx.waker().clone()),
        ) {
            println!("GetStateKeys Result ready for entry: {}", self.entry_index);
            let result = get_state_keys_entry_message::StateKeys::decode(result).unwrap();
            Poll::Ready(result.keys)
        } else {
            println!("GetStateKeys Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct SetStateFuture {
    entry_index: u32,
    entry: SetStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(SetStateFuture, SetStateEntry);

impl Future for SetStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::SetState(self.entry.clone()),
            None,
        );
        println!("SetState Result ready for entry: {}", self.entry_index);
        Poll::Ready(())
    }
}

pub struct ClearStateFuture {
    entry_index: u32,
    entry: ClearStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(ClearStateFuture, ClearStateEntry);

impl Future for ClearStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::ClearState(self.entry.clone()),
            None,
        );
        println!("ClearState Result ready for entry: {}", self.entry_index);
        Poll::Ready(())
    }
}

pub struct ClearAllStateFuture {
    entry_index: u32,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl ClearAllStateFuture {
    pub fn new(state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            state_machine,
        }
    }

    pub fn entry(&self) -> u32 {
        self.entry_index
    }
}

impl Future for ClearAllStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.state_machine
            .lock()
            .handle_user_code_message(self.entry_index, Entry::ClearAllState, None);
        println!("ClearAllState Result ready for entry: {}", self.entry_index);
        Poll::Ready(())
    }
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
            Some(cx.waker().clone()),
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
            Some(cx.waker().clone()),
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
            Some(cx.waker().clone()),
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
            Some(cx.waker().clone()),
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
            Some(cx.waker().clone()),
        ) {
            println!("GetPromise Result ready for entry: {}", self.entry_index);
            Poll::Ready(result)
        } else {
            println!("GetPromise Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct PeekPromiseFuture {
    entry_index: u32,
    entry: PeekPromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(PeekPromiseFuture, PeekPromiseEntry);

impl Future for PeekPromiseFuture {
    type Output = Option<Bytes>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::PeekPromise(self.entry.clone()),
            Some(cx.waker().clone()),
        ) {
            println!("PeekPromise Result ready for entry: {}", self.entry_index);
            if !result.is_empty() {
                Poll::Ready(Some(result))
            } else {
                Poll::Ready(None)
            }
        } else {
            println!("PeekPromise Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

pub struct CompletePromiseFuture {
    entry_index: u32,
    entry: CompletePromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
}

future_impl!(CompletePromiseFuture, CompletePromiseEntry);

impl Future for CompletePromiseFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(_) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Entry::CompletePromise(self.entry.clone()),
            Some(cx.waker().clone()),
        ) {
            println!("CompletePromise Result ready for entry: {}", self.entry_index);
            Poll::Ready(())
        } else {
            println!("CompletePromise Result pending for entry: {}", self.entry_index);
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_call_service() {}
}
