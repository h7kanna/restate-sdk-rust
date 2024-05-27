use crate::{
    machine::StateMachine,
    syscall::{CallService, SleepService},
};
use futures_util::FutureExt;
use parking_lot::Mutex;
use restate_sdk_core::ServiceHandler;
use restate_sdk_types::journal::{InvokeEntry, InvokeRequest, SleepEntry};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    ops::Add,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub enum CallContextType {
    None,
    Run,
}

#[derive(Clone)]
pub struct RestateContext {
    state_machine: Arc<Mutex<StateMachine>>,
}

impl RestateContext {
    pub(crate) fn new(state_machine: Arc<Mutex<StateMachine>>) -> Self {
        RestateContext { state_machine }
    }

    pub fn sleep(&self, timeout_millis: u64) -> impl Future<Output = Result<(), anyhow::Error>> + '_ {
        let wake_up_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let wake_up_time = wake_up_time.as_millis() as u64 + timeout_millis;
        println!("Context sleep: Wake up time {}", wake_up_time);
        async move {
            let _ = SleepService::new(
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
        let result = serde_json::to_string(&parameter).unwrap();
        async move {
            let bytes = CallService::<String>::new(
                InvokeEntry {
                    request: InvokeRequest {
                        service_name: service_name.into(),
                        handler_name: handler_name.into(),
                        parameter: result.into(),
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
