use crate::{
    machine::StateMachine,
    protocol::AWAKEABLE_IDENTIFIER_PREFIX,
    syscall::{AwakeableFuture, CallServiceFuture, RunFuture, SleepFuture},
    utils,
};
use base64::Engine;
use bytes::{BufMut, Bytes, BytesMut};
use futures_util::FutureExt;
use parking_lot::Mutex;
use restate_sdk_core::{RunAction, ServiceHandler};
use restate_sdk_types::journal::{
    AwakeableEntry, EntryResult, InvokeEntry, InvokeRequest, RunEntry, SleepEntry,
};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    ops::Add,
    sync::Arc,
    time::{Duration, SystemTime},
};

#[derive(Clone)]
pub struct Request {
    pub id: Bytes,
}

pub enum CallContextType {
    None,
    Run,
}

#[derive(Clone)]
pub struct RestateContext {
    request: Request,
    state_machine: Arc<Mutex<StateMachine>>,
}

impl RestateContext {
    pub(crate) fn new(request: Request, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        RestateContext {
            request,
            state_machine,
        }
    }

    pub fn awakeable<R>(&self) -> (String, impl Future<Output = Result<R, anyhow::Error>> + '_)
    where
        for<'a> R: Serialize + Deserialize<'a>,
    {
        let awakeable = AwakeableFuture::new(AwakeableEntry { result: None }, self.state_machine.clone());
        let mut input_buf = BytesMut::new();
        input_buf.put_slice(&self.request.id);
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

    pub fn sleep(&self, timeout_millis: u64) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        let wake_up_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let wake_up_time = wake_up_time.as_millis() as u64 + timeout_millis;
        println!("Context sleep: Wake up time {}", wake_up_time);
        async move {
            let _ = SleepFuture::new(
                SleepEntry {
                    wake_up_time,
                    result: None,
                },
                self.state_machine.clone(),
            )
            .await;
            Ok(())
        }
    }

    pub fn run<F, R>(&self, func: F) -> impl Future<Output = Result<(), anyhow::Error>> + '_
    where
        for<'a> R: Serialize + Deserialize<'a>,
        F: RunAction<Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
    {
        async move {
            let _ = RunFuture::new(
                RunEntry {
                    result: EntryResult::Success(Bytes::new()),
                },
                self.state_machine.clone(),
            );
            let result = func().await;
            Ok(())
        }
    }

    pub fn invoke<F, I, R>(
        &self,
        _func: F,
        service_name: String,
        handler_name: String,
        parameter: I,
        key: Option<String>,
    ) -> impl Future<Output = Result<R, anyhow::Error>> + '_
    where
        for<'a> I: Serialize + Deserialize<'a>,
        for<'a> R: Serialize + Deserialize<'a>,
        F: ServiceHandler<RestateContext, I, Output = Result<R, anyhow::Error>> + Send + Sync + 'static,
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
                self.state_machine.clone(),
            )
            .await;

            // If the system call is completed, deserialize the result and return
            let bytes = bytes.to_vec();
            let result: R = serde_json::from_slice(&bytes).unwrap();
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_context() {}
}
