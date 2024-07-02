use bytes::Bytes;
use http_body_util::BodyExt;
use restate_sdk_types::{
    journal::{
        ClearStateEntry, CompletionResult, GetStateEntry, GetStateKeysEntry, GetStateKeysResult,
        SetStateEntry,
    },
    service_protocol::start_message,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct LocalStateStore {
    is_partial: bool,
    state: HashMap<String, Option<Bytes>>,
}

impl LocalStateStore {
    pub fn new(is_partial: bool, state: Vec<start_message::StateEntry>) -> Self {
        Self {
            is_partial,
            state: state
                .into_iter()
                .map(|entry| (String::from_utf8(entry.key.to_vec()).unwrap(), Some(entry.value)))
                .collect(),
        }
    }

    pub fn try_complete_get(&self, key: &String, message: &mut GetStateEntry) -> bool {
        let state_entry = self.state.get(key);
        return if let Some(state_entry) = state_entry {
            if let Some(bytes) = state_entry {
                message.value = Some(CompletionResult::Success(bytes.clone()));
            } else {
                message.value = Some(CompletionResult::Empty);
            }
            true
        } else {
            if self.is_partial {
                false
            } else {
                message.value = Some(CompletionResult::Empty);
                true
            }
        };
    }

    pub fn try_complete_get_keys(&self, message: &mut GetStateKeysEntry) -> bool {
        if self.is_partial {
            return false;
        }
        message.value = Some(GetStateKeysResult::Result(
            self.state
                .keys()
                .into_iter()
                .map(|key| key.clone().into())
                .collect(),
        ));
        return true;
    }

    pub fn set<V>(&mut self, key: String, value: V) -> SetStateEntry
    where
        for<'a> V: Serialize + Deserialize<'a>,
    {
        let bytes = serde_json::to_string(&value).unwrap();
        let bytes: Bytes = bytes.into();
        self.state.insert(key.clone(), Some(bytes.clone()));
        SetStateEntry {
            key: key.into(),
            value: bytes,
        }
    }

    pub fn add(&mut self, key: String, value: Bytes) {
        self.state.insert(key.clone(), Some(value));
    }

    pub fn clear(&mut self, key: String) -> ClearStateEntry {
        self.state.insert(key.clone(), None);
        ClearStateEntry { key: key.into() }
    }

    pub fn clear_all(&mut self) {
        self.state.clear();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_store() {}
}
