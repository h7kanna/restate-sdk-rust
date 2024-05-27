use bytes::Bytes;
use restate_sdk_types::service_protocol::start_message;
use std::collections::HashMap;

pub struct LocalStore {
    is_partial: bool,
    state: HashMap<String, Option<Bytes>>,
}

impl LocalStore {
    pub fn new(is_partial: bool, state: Vec<start_message::StateEntry>) -> Self {
        Self {
            is_partial,
            state: state
                .into_iter()
                .map(|entry| (String::from_utf8(entry.key.to_vec()).unwrap(), Some(entry.value)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_store() {}
}
