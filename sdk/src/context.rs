use crate::{machine::StateMachine, syscall::CallService};
use bytes::Bytes;
use parking_lot::Mutex;
use restate_sdk_service_protocol::CallEntryMessage;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};

pub enum CallContextType {
    None,
    Run,
}

pub struct RestateContext {
    state_machine: Arc<Mutex<StateMachine>>,
}

impl RestateContext {
    pub(crate) fn new(state_machine: Arc<Mutex<StateMachine>>) -> Self {
        RestateContext { state_machine }
    }

    pub fn invoke_service<F, I, R>(&self, func: F, input: I) -> impl Future<Output = R> + '_
    where
        I: Serialize,
        for<'a> R: Deserialize<'a>,
    {
        self.invoke(
            "test".to_string(),
            "test".to_string(),
            "greet".to_string(),
            input,
            None,
        )
    }

    pub fn invoke<F, I, R>(
        &self,
        func: F,
        service_name: String,
        handler_name: String,
        parameter: I,
        key: Option<String>,
    ) -> impl Future<Output = R> + '_
    where
        I: Serialize,
        for<'a> R: Deserialize<'a>,
    {
        async move {
            let bytes = CallService::<String>::new(
                CallEntryMessage {
                    service_name,
                    handler_name,
                    parameter: Bytes::new(),
                    headers: vec![],
                    key: "".to_string(),
                    name: "".to_string(),
                    result: None,
                },
                self.state_machine.clone(),
            )
            .await;
            // If the system call is completed, deserialize the result and return
            let bytes = bytes.to_vec();
            let result: R = serde_json::from_slice(&bytes).unwrap();
            result
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_context() {}
}
