use crate::machine::StateMachine;
use bytes::Bytes;
use parking_lot::Mutex;
use restate_sdk_protos::CallEntryMessage;
use restate_sdk_types::protocol::{Message, INVOKE_ENTRY_MESSAGE_TYPE};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct CallService<T> {
    entry_index: u32,
    call_message: CallEntryMessage,
    state_machine: Arc<Mutex<StateMachine>>,
    _ret: PhantomData<T>,
}

impl<T> CallService<T> {
    pub fn new(call_message: CallEntryMessage, state_machine: Arc<Mutex<StateMachine>>) -> Self {
        let entry_index = state_machine.lock().get_next_user_code_journal_index();
        Self {
            entry_index,
            call_message,
            state_machine,
            _ret: PhantomData,
        }
    }
}

impl<T> Future for CallService<T> {
    type Output = Bytes;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.state_machine.lock().handle_user_code_message(
            self.entry_index,
            Message::CallEntryMessage(INVOKE_ENTRY_MESSAGE_TYPE, self.call_message.clone()),
            cx.waker().clone(),
        ) {
            Poll::Ready(result)
        } else {
            println!("I am here");
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_call_service() {}
}
