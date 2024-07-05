use crate::{
    machine::StateMachine,
    protocol::AWAKEABLE_IDENTIFIER_PREFIX,
    syscall::{
        AwakeableFuture, CallServiceFuture, ClearAllStateFuture, ClearStateFuture, CompletePromiseFuture,
        GetPromiseFuture, GetStateFuture, GetStateKeysFuture, PeekPromiseFuture, RunFuture, SetStateFuture,
        SleepFuture,
    },
    utils,
};
use anyhow::Error;
use base64::Engine;
use bytes::{BufMut, Bytes, BytesMut};
use futures_util::FutureExt;
use parking_lot::Mutex;
use restate_sdk_core::{RunAction, ServiceHandler};
use restate_sdk_types::journal::{
    AwakeableEntry, CompletePromiseEntry, EntryResult, GetPromiseEntry, GetStateEntry, GetStateKeysEntry,
    InvokeEntry, InvokeRequest, PeekPromiseEntry, RunEntry, SleepEntry,
};
use serde::{Deserialize, Serialize};
use std::{
    future,
    future::Future,
    marker::PhantomData,
    ops::Add,
    pin::Pin,
    sync::Arc,
    task::Poll,
    time::{Duration, SystemTime},
};
use tracing::{info, Instrument};

#[derive(Clone)]
pub struct Request {
    pub id: Bytes,
}

pub enum CallContextType {
    None,
    Run,
}

pub trait ContextDate {
    fn now(&self) -> impl Future<Output = u64>;
    fn to_json(&self) -> impl Future<Output = String>;
}

pub trait ContextData {
    fn request(&self) -> &Request;
}

pub(crate) trait ContextInstance: ContextData {
    fn new(request: Request, state_machine: Arc<Mutex<StateMachine>>) -> Self;
    fn state_machine(&self) -> Arc<Mutex<StateMachine>>;
}

macro_rules! context_data_impl {
    ($test:ident) => {
        impl ContextData for $test {
            fn request(&self) -> &Request {
                &self.request
            }
        }

        impl ContextInstance for $test {
            fn new(request: Request, state_machine: Arc<Mutex<StateMachine>>) -> Self {
                $test {
                    request,
                    state_machine,
                }
            }

            fn state_machine(&self) -> Arc<Mutex<StateMachine>> {
                self.state_machine.clone()
            }
        }
    };
}

pub trait ContextBase: ContextInstance {
    fn awakeable<R>(&self) -> (String, impl Future<Output = Result<R, Error>> + '_)
    where
        for<'a> R: Serialize + Deserialize<'a>,
    {
        let awakeable = AwakeableFuture::new(AwakeableEntry { result: None }, self.state_machine().clone());
        let mut input_buf = BytesMut::new();
        input_buf.put_slice(&self.request().id);
        input_buf.put_u32(awakeable.entry());
        let encoded_base64 = utils::base64::URL_SAFE.encode(input_buf.freeze());
        let id = format!("{}{}", AWAKEABLE_IDENTIFIER_PREFIX, encoded_base64);
        (id, async move {
            let bytes = awakeable.await;
            // If the awakeable is completed, deserialize the result and return
            let bytes = bytes.to_vec();
            let result: R = serde_json::from_slice(&bytes).unwrap();
            Ok(result)
        })
    }

    fn sleep(&self, timeout_millis: u64) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        let wake_up_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let wake_up_time = wake_up_time.as_millis() as u64 + timeout_millis;
        info!("Context sleep: Wake up time {}", wake_up_time);
        async move {
            let _ = SleepFuture::new(
                SleepEntry {
                    wake_up_time,
                    result: None,
                },
                self.state_machine().clone(),
            )
            .await;
            Ok(())
        }
    }

    fn run<Func, Output>(&self, func: Func) -> impl Future<Output = Result<(), anyhow::Error>> + '_
    where
        for<'a> Output: Serialize + Deserialize<'a>,
        Func: RunAction<Output = Result<Output, anyhow::Error>> + Send + Sync + 'static,
    {
        async move {
            let _ = RunFuture::new(
                RunEntry {
                    result: EntryResult::Success(Bytes::new()),
                },
                self.state_machine().clone(),
            );
            let result = func().await;
            Ok(())
        }
    }

    fn invoke<Context, Func, Input, Output>(
        &self,
        _func: Func,
        service_name: String,
        handler_name: String,
        parameter: Input,
        key: Option<String>,
    ) -> impl Future<Output = Result<Output, anyhow::Error>> + '_
    where
        for<'a> Input: Serialize + Deserialize<'a>,
        for<'a> Output: Serialize + Deserialize<'a>,
        Func: ServiceHandler<Context, Input, Output = Result<Output, anyhow::Error>> + Send + Sync + 'static,
        Context: ContextInstance,
    {
        let parameter = serde_json::to_string(&parameter).unwrap();
        async move {
            let bytes = CallServiceFuture::<String>::new(
                InvokeEntry {
                    request: InvokeRequest {
                        service_name: service_name.into(),
                        handler_name: handler_name.into(),
                        parameter: parameter.into(),
                        key: Default::default(),
                    },
                    result: None,
                },
                self.state_machine().clone(),
            )
            .await;

            // If the system call is completed, deserialize the result and return
            let bytes = bytes.to_vec();
            let result: Output = serde_json::from_slice(&bytes).unwrap();
            Ok(result)
        }
    }
}

pub trait ContextKeyed: ContextData {
    fn key() -> String {
        "".to_string()
    }
}

pub trait KeyValueStoreReadOnly: ContextInstance {
    fn get<V, N>(&self, name: N) -> impl Future<Output = Option<V>>
    where
        for<'a> V: Serialize + Deserialize<'a>,
        N: AsRef<str>,
    {
        let mut get_state_entry = GetStateEntry {
            key: name.as_ref().to_string().into(),
            value: None,
        };
        let completed = self
            .state_machine()
            .lock()
            .local_state_store()
            .try_complete_get(name.as_ref(), &mut get_state_entry);
        let get_state = GetStateFuture::new(get_state_entry, self.state_machine().clone());
        let state_machine = self.state_machine().clone();
        async move {
            let bytes = get_state.await;
            if !completed {
                state_machine
                    .lock()
                    .local_state_store()
                    .add(name.as_ref().to_string(), bytes.clone());
            }
            let bytes = bytes.to_vec();
            let result: V = serde_json::from_slice(&bytes).unwrap();
            Some(result)
        }
    }

    fn state_keys<V>(&self) -> impl Future<Output = Vec<V>>
    where
        for<'a> V: Serialize + Deserialize<'a>,
    {
        let mut get_state_keys_entry = GetStateKeysEntry { value: None };
        let completed = self
            .state_machine()
            .lock()
            .local_state_store()
            .try_complete_get_keys(&mut get_state_keys_entry);
        let get_state_keys = GetStateKeysFuture::new(get_state_keys_entry, self.state_machine().clone());
        async move {
            let bytes = get_state_keys.await;
            bytes.iter().map(|v| serde_json::from_slice(v).unwrap()).collect()
        }
    }
}

pub trait KeyValueStore: KeyValueStoreReadOnly {
    fn set<V, N>(&self, name: N, value: V) -> impl Future<Output = ()>
    where
        for<'a> V: Serialize + Deserialize<'a>,
        N: AsRef<str>,
    {
        let set_state_entry = self
            .state_machine()
            .lock()
            .local_state_store()
            .set(name.as_ref().to_string(), value);
        SetStateFuture::new(set_state_entry, self.state_machine().clone())
    }

    fn clear<N: AsRef<str>>(&self, name: N) -> impl Future<Output = ()> {
        let clear_state_entry = self
            .state_machine()
            .lock()
            .local_state_store()
            .clear(name.as_ref().to_string());
        ClearStateFuture::new(clear_state_entry, self.state_machine().clone())
    }

    fn clear_all(&self) -> impl Future<Output = ()> {
        self.state_machine().lock().local_state_store().clear_all();
        ClearAllStateFuture::new(self.state_machine().clone())
    }
}

#[derive(Clone)]
pub struct Context {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

context_data_impl!(Context);

impl ContextBase for Context {}

#[derive(Clone)]
pub struct ObjectSharedContext {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

context_data_impl!(ObjectSharedContext);

impl ContextBase for ObjectSharedContext {}

impl ContextKeyed for ObjectSharedContext {}

impl KeyValueStoreReadOnly for ObjectSharedContext {}

#[derive(Clone)]
pub struct ObjectContext {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

context_data_impl!(ObjectContext);

impl ContextBase for ObjectContext {}

impl ContextKeyed for ObjectContext {}

impl KeyValueStoreReadOnly for ObjectContext {}

impl KeyValueStore for ObjectContext {}

pub trait CombinablePromise<T: Send>: Future<Output = T> + Send {
    fn or_timeout(&self, millis: u64) -> impl Future<Output = T> + Send;
}

pub trait DurablePromise {
    fn peek<T: Send>(&self) -> impl Future<Output = Option<T>> + Send
    where
        for<'a> T: Serialize + Deserialize<'a>;
    fn resolve<T: Send>(&self, value: T) -> impl Future<Output = ()> + Send
    where
        for<'a> T: Serialize + Deserialize<'a>;
    fn reject(&self, message: String) -> impl Future<Output = ()> + Send;
    fn get<T: Send>(&self) -> impl CombinablePromise<T>
    where
        for<'a> T: Serialize + Deserialize<'a>;
    fn awaitable<T: Send>(&self) -> impl Future<Output = T> + Send
    where
        for<'a> T: Serialize + Deserialize<'a>;
}

pub trait ContextWorkflowShared: ContextInstance {
    fn promise<N: AsRef<str>>(&self, name: N) -> impl DurablePromise {
        DurablePromiseImpl::new(name.as_ref().to_string(), self.state_machine().clone())
    }
}

#[derive(Clone)]
pub struct WorkflowSharedContext {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

context_data_impl!(WorkflowSharedContext);

impl ContextWorkflowShared for WorkflowSharedContext {}

#[derive(Clone)]
pub struct WorkflowContext {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

context_data_impl!(WorkflowContext);

impl ContextBase for WorkflowContext {}

impl ContextKeyed for WorkflowContext {}

impl KeyValueStoreReadOnly for WorkflowContext {}

impl KeyValueStore for WorkflowContext {}

impl ContextWorkflowShared for WorkflowContext {}

pub struct CombinablePromiseImpl<T> {
    entry_index: u32,
    state_machine: Arc<Mutex<StateMachine>>,
    _ret: PhantomData<T>,
}

impl<T: Send> CombinablePromiseImpl<T> {
    pub fn new(state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            state_machine,
            _ret: PhantomData,
        }
    }
}

impl<T: Send> Future for CombinablePromiseImpl<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::<T>::Pending
    }
}

impl<T: Send> CombinablePromise<T> for CombinablePromiseImpl<T> {
    fn or_timeout(&self, millis: u64) -> impl Future<Output = T> {
        async { future::pending::<T>().await }
    }
}

pub struct DurablePromiseImpl {
    name: String,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl DurablePromiseImpl {
    pub fn new(name: String, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        Self { name, state_machine }
    }
}

impl DurablePromise for DurablePromiseImpl {
    fn peek<T: Send>(&self) -> impl Future<Output = Option<T>>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        let peek_promise = PeekPromiseFuture::new(
            PeekPromiseEntry {
                key: self.name.clone().into(),
                value: None,
            },
            self.state_machine.clone(),
        );

        async move {
            let bytes = peek_promise.in_current_span().await;
            bytes.map(|bytes| {
                // If the system call is completed, deserialize the result and return
                let bytes = bytes.to_vec();
                let result: T = serde_json::from_slice(&bytes).unwrap();
                result
            })
        }
    }

    fn resolve<T: Send>(&self, value: T) -> impl Future<Output = ()>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        let value = serde_json::to_string(&value).unwrap();
        CompletePromiseFuture::new(
            CompletePromiseEntry {
                key: self.name.clone().into(),
                completion: EntryResult::Success(value.into()),
                value: None,
            },
            self.state_machine.clone(),
        )
    }

    fn reject(&self, message: String) -> impl Future<Output = ()> {
        CompletePromiseFuture::new(
            CompletePromiseEntry {
                key: self.name.clone().into(),
                completion: EntryResult::Failure(0u32.into(), message.into()),
                value: None,
            },
            self.state_machine.clone(),
        )
    }

    fn get<T: Send>(&self) -> impl CombinablePromise<T>
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        CombinablePromiseImpl::new(self.state_machine.clone())
    }

    fn awaitable<T: Send>(&self) -> impl Future<Output = T> + Send
    where
        for<'a> T: Serialize + Deserialize<'a>,
    {
        let get_promise = GetPromiseFuture::new(
            GetPromiseEntry {
                key: self.name.clone().into(),
                value: None,
            },
            self.state_machine.clone(),
        );
        async move {
            let bytes = get_promise.in_current_span().await;
            // If the system call is completed, deserialize the result and return
            let bytes = bytes.to_vec();
            let result: T = serde_json::from_slice(&bytes).unwrap();
            result
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_context() {}
}
