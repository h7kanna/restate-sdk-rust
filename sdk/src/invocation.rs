use crate::connection::RestateStreamConsumer;
use dashmap::DashMap;
use restate_sdk_types::Message;

enum State {
    ExpectingStart = 0,
    ExpectingInput = 1,
    ExpectingFurtherReplay = 2,
    Complete = 3,
}

pub(crate) struct Invocation {
    pub nb_entries_to_replay: u32,
    pub replay_entries: DashMap<u32, Message>,
}

pub(crate) struct InvocationBuilder {
    state: State,
}

impl RestateStreamConsumer for &mut InvocationBuilder {
    async fn handle(&mut self, message: Message) -> bool {
        match self.state {
            State::ExpectingStart => {}
            State::ExpectingInput => {}
            State::ExpectingFurtherReplay => {}
            State::Complete => {}
        }
        true
    }
}

impl InvocationBuilder {
    pub fn new() -> Self {
        Self {
            state: State::ExpectingStart,
        }
    }

    pub fn build(self) -> Invocation {
        Invocation {
            nb_entries_to_replay: 0,
            replay_entries: DashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_invocation() {}
}
