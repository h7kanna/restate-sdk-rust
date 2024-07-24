use crate::machine::StateMachine;
use bytes::Bytes;
use futures_util::FutureExt;
use parking_lot::{Mutex, MutexGuard};
use pin_project::pin_project;
use prost::Message;
use restate_sdk_types::{
    journal::{
        AwakeableEntry, ClearStateEntry, CompletePromiseEntry, Entry, GetPromiseEntry, GetStateEntry,
        GetStateKeysEntry, InvokeEntry, PeekPromiseEntry, RunEntry, SetStateEntry, SleepEntry,
    },
    service_protocol,
    service_protocol::{get_state_keys_entry_message, CombinatorEntryMessage},
};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::SystemTime,
};
use tracing::{debug, info};

pub trait JournalIndex {
    fn entry_index(&self) -> u32;
}

macro_rules! future_impl {
    ($future:ident, $entry:ident) => {
        impl JournalIndex for $future {
            fn entry_index(&self) -> u32 {
                self.entry_index.load(Ordering::Relaxed)
            }
        }
        impl $future {
            pub fn new(entry: $entry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
                Self {
                    entry,
                    state_machine,
                    entry_index: Arc::new(AtomicU32::new(0)),
                    polled: Arc::new(AtomicBool::new(false)),
                }
            }

            fn set_span(&self, mut state_machine: MutexGuard<'_, StateMachine>) {
                state_machine.set_span()
            }
        }
    };
}

pub struct GetStateFuture {
    entry: GetStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(GetStateFuture, GetStateEntry);

impl Future for GetStateFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::GetState(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("GetState Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(result)
        } else {
            debug!("GetState Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct GetStateKeysFuture {
    entry: GetStateKeysEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(GetStateKeysFuture, GetStateKeysEntry);

impl Future for GetStateKeysFuture {
    type Output = Vec<Bytes>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::GetStateKeys(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("GetStateKeys Result ready for entry: {}", entry_index);
            let result = get_state_keys_entry_message::StateKeys::decode(result).unwrap();
            self.set_span(state_machine);
            Poll::Ready(result.keys)
        } else {
            debug!("GetStateKeys Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct SetStateFuture {
    entry: SetStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(SetStateFuture, SetStateEntry);

impl Future for SetStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        self.state_machine.lock().handle_user_code_message(
            entry_index,
            Entry::SetState(self.entry.clone()),
            None,
        );
        debug!("SetState Result ready for entry: {:?}", entry_index);
        Poll::Ready(())
    }
}

pub struct ClearStateFuture {
    entry: ClearStateEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    polled: Arc<AtomicBool>,
    entry_index: Arc<AtomicU32>,
}

future_impl!(ClearStateFuture, ClearStateEntry);

impl Future for ClearStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        self.state_machine.lock().handle_user_code_message(
            entry_index,
            Entry::ClearState(self.entry.clone()),
            None,
        );
        debug!("ClearState Result ready for entry: {:?}", entry_index);
        Poll::Ready(())
    }
}

pub struct ClearAllStateFuture {
    state_machine: Arc<Mutex<StateMachine>>,
    polled: Arc<AtomicBool>,
    entry_index: Arc<AtomicU32>,
}

impl ClearAllStateFuture {
    pub fn new(state_machine: Arc<Mutex<StateMachine>>) -> Self {
        Self {
            state_machine,
            entry_index: Arc::new(AtomicU32::new(0)),
            polled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn entry(&self) -> u32 {
        self.entry_index.load(Ordering::Relaxed)
    }
}

impl Future for ClearAllStateFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        self.state_machine
            .lock()
            .handle_user_code_message(entry_index, Entry::ClearAllState, None);
        debug!("ClearAllState Result ready for entry: {:?}", entry_index);
        Poll::Ready(())
    }
}

pub struct AwakeableFuture {
    entry: AwakeableEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

//future_impl!(AwakeableFuture, AwakeableEntry);

impl AwakeableFuture {
    pub fn new(entry: AwakeableEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry,
            state_machine,
            entry_index: Arc::new(AtomicU32::new(entry_index)),
            polled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn entry(&self) -> u32 {
        self.entry_index.load(Ordering::Relaxed)
    }

    fn set_span(&self, mut state_machine: MutexGuard<'_, StateMachine>) {
        state_machine.set_span()
    }
}

impl Future for AwakeableFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::Awakeable(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("Run Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(result)
        } else {
            debug!("Run Result pending for entry: {}", entry_index);
            //self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct SleepFuture {
    entry: SleepEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(SleepFuture, SleepEntry);

impl Future for SleepFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        debug!("Sleep future polling");
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::Sleep(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("Sleep Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(result)
        } else {
            debug!("Sleep Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct RunFuture {
    entry: RunEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(RunFuture, RunEntry);

impl Future for RunFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::Run(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("Run Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(result)
        } else {
            debug!("Run Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct CallServiceFuture<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    invoke_entry: InvokeEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
    _ret: PhantomData<T>,
}

impl<T> CallServiceFuture<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    pub fn new(invoke_entry: InvokeEntry, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        Self {
            invoke_entry,
            state_machine,
            entry_index: Arc::new(AtomicU32::new(0)),
            polled: Arc::new(AtomicBool::new(false)),
            _ret: PhantomData,
        }
    }

    fn set_span(&self, mut state_machine: MutexGuard<'_, StateMachine>) {
        state_machine.set_span()
    }
}

impl<T> JournalIndex for CallServiceFuture<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    fn entry_index(&self) -> u32 {
        self.entry_index.load(Ordering::Relaxed)
    }
}

impl<T> Future for CallServiceFuture<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    type Output = Result<T, anyhow::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        debug!("Call future polling");
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::Call(self.invoke_entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("Call Result ready for entry: {}", entry_index);
            self.set_span(state_machine);

            let result: T = serde_json::from_slice(&result).unwrap();
            Poll::Ready(Ok(result))
        } else {
            debug!("Call Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct GetPromiseFuture {
    entry: GetPromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(GetPromiseFuture, GetPromiseEntry);

impl Future for GetPromiseFuture {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::GetPromise(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("GetPromise Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(result)
        } else {
            debug!("GetPromise Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct PeekPromiseFuture {
    entry: PeekPromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(PeekPromiseFuture, PeekPromiseEntry);

impl Future for PeekPromiseFuture {
    type Output = Option<Bytes>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::PeekPromise(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(result) = result {
            debug!("PeekPromise Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            if !result.is_empty() {
                Poll::Ready(Some(result))
            } else {
                Poll::Ready(None)
            }
        } else {
            debug!("PeekPromise Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

pub struct CompletePromiseFuture {
    entry: CompletePromiseEntry,
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
}

future_impl!(CompletePromiseFuture, CompletePromiseEntry);

impl Future for CompletePromiseFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_machine = self.state_machine.lock();
        let entry_index = if self.polled.fetch_or(true, Ordering::Relaxed) {
            Some(self.entry_index.load(Ordering::Relaxed))
        } else {
            None
        };
        let (entry_index, result) = state_machine.handle_user_code_message(
            entry_index,
            Entry::CompletePromise(self.entry.clone()),
            Some(cx.waker().clone()),
        );
        if let Some(_) = result {
            debug!("CompletePromise Result ready for entry: {}", entry_index);
            self.set_span(state_machine);
            Poll::Ready(())
        } else {
            debug!("CompletePromise Result pending for entry: {}", entry_index);
            self.entry_index.store(entry_index, Ordering::Relaxed);
            state_machine.abort_on_replay();
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_call_service() {}
}
