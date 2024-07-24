use crate::{context::JournalIndex, machine::StateMachine, syscall::SleepFuture};
use anyhow::anyhow;
use parking_lot::{Mutex, MutexGuard};
use prost::Message;
use restate_sdk_types::{
    journal::{Entry, SleepEntry},
    service_protocol::CombinatorEntryMessage,
};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::SystemTime,
};
use tracing::{debug, info};

#[pin_project::pin_project]
pub struct Timeout<F>
where
    F: Future + JournalIndex,
{
    state_machine: Arc<Mutex<StateMachine>>,
    entry_index: Arc<AtomicU32>,
    polled: Arc<AtomicBool>,
    #[pin]
    future: F,
    #[pin]
    timer: SleepFuture,
}

impl<F> Timeout<F>
where
    F: Future + JournalIndex,
{
    pub fn new(state_machine: Arc<Mutex<StateMachine>>, timeout_millis: u64, future: F) -> Self {
        let wake_up_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let wake_up_time = wake_up_time.as_millis() as u64 + timeout_millis;
        info!("Context sleep: Wake up time {}", wake_up_time);
        let timer = SleepFuture::new(
            SleepEntry {
                wake_up_time,
                result: None,
            },
            state_machine.clone(),
        );
        Self {
            state_machine,
            entry_index: Arc::new(AtomicU32::new(0)),
            polled: Arc::new(AtomicBool::new(false)),
            future,
            timer,
        }
    }

    fn set_span(&self, mut state_machine: MutexGuard<'_, StateMachine>) {
        state_machine.set_span()
    }
}

impl<F> Future for Timeout<F>
where
    F: Future + JournalIndex,
{
    type Output = Result<F::Output, anyhow::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut selfp = self.project();
        let result = selfp.future.as_mut().poll(cx);
        let timeout = selfp.timer.as_mut().poll(cx);

        if result.is_pending() && timeout.is_pending() {
            Poll::Pending
        } else {
            let mut state_machine = selfp.state_machine.lock();
            let future_index = selfp.future.entry_index();
            let timer_index = selfp.timer.entry_index();

            let entry_index = if selfp.polled.fetch_or(true, Ordering::Relaxed) {
                Some(selfp.entry_index.load(Ordering::Relaxed))
            } else {
                None
            };

            let journal_entries_order = if result.is_pending() && timeout.is_ready() {
                debug!("Timeout for timer entry: {:?}", timer_index);
                //vec![timer_index as i32, future_index as i32]
                vec![timer_index as i32]
            } else {
                vec![future_index as i32, timer_index as i32]
            };

            let entry = Entry::Custom(
                CombinatorEntryMessage {
                    combinator_id: 0,
                    journal_entries_order,
                }
                .encode_to_vec()
                .into(),
            );

            let (entry_index, done) =
                state_machine.write_combinator_order(entry_index, entry, cx.waker().clone());
            if let Some(_) = done {
                debug!("Timeout Result ready for entry: {}", entry_index);
                //selfp.set_span(state_machine);
                if timeout.is_ready() {
                    Poll::Ready(Err(anyhow!("Timeout")))
                } else {
                    result.map(|result| Ok(result))
                }
            } else {
                debug!("Timeout Result pending for entry: {}", entry_index);
                selfp.entry_index.store(entry_index, Ordering::Relaxed);
                state_machine.abort_on_replay();
                Poll::Pending
            }
        }
    }
}
