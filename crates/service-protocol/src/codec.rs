// Copyright (c) 2023 -  Restate Software, Inc., Restate GmbH.
// All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::Message;
use restate_sdk_types::{
    invocation::Header,
    journal::{
        raw::{ErrorKind, PlainEntryHeader, PlainRawEntry, RawEntry, RawEntryCodec, RawEntryCodecError},
        CompletionResult, Entry, EntryType,
    },
    service_protocol,
};
use std::{fmt::Debug, mem};

/// This macro generates the pattern matching with arms per entry.
/// For each entry it first executes `Message#decode` and then `try_into()`.
/// It expects that for each `{...}Entry` there is a valid `TryFrom<{...}Message>` implementation with `Error = &'static str`.
/// These implementations are available in [`super::pb_into`].
macro_rules! match_decode {
    ($ty:expr, $buf:expr, { $($variant:ident),* }) => {
        match $ty {
              $(EntryType::$variant { .. } => paste::paste! {
                  service_protocol::[<$variant EntryMessage>]::decode($buf)
                    .map_err(|e| RawEntryCodecError::new($ty.clone(), ErrorKind::Decode { source: Some(e.into()) }))
                    .and_then(|msg| msg.try_into().map_err(|f| RawEntryCodecError::new($ty.clone(), ErrorKind::MissingField(f))))
              },)*
             EntryType::Custom => Ok(Entry::Custom($buf.copy_to_bytes($buf.remaining()))),
        }
    };
}

#[derive(Clone, PartialEq, ::prost::Message)]
struct NamedEntryTemplate {
    // By spec the field `name` is always tag 12
    #[prost(string, optional, tag = "12")]
    name: Option<String>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ProtobufRawEntryCodec;

impl RawEntryCodec for ProtobufRawEntryCodec {
    fn serialize_as_input_entry(headers: Vec<Header>, value: Bytes) -> PlainRawEntry {
        RawEntry::new(
            PlainEntryHeader::Input {},
            service_protocol::InputEntryMessage {
                headers: headers
                    .into_iter()
                    .map(|h| service_protocol::Header {
                        key: h.name.to_string(),
                        value: h.value.to_string(),
                    })
                    .collect(),
                value,
                ..Default::default()
            }
            .encode_to_vec()
            .into(),
        )
    }

    fn serialize_get_state_keys_completion(keys: Vec<Bytes>) -> CompletionResult {
        CompletionResult::Success(
            service_protocol::get_state_keys_entry_message::StateKeys { keys }
                .encode_to_vec()
                .into(),
        )
    }

    fn deserialize(entry_type: EntryType, mut entry_value: Bytes) -> Result<Entry, RawEntryCodecError> {
        // We clone the entry Bytes here to ensure that the generated Message::decode
        // invocation reuses the same underlying byte array.
        match_decode!(entry_type, entry_value, {
            Input,
            Output,
            GetState,
            SetState,
            ClearState,
            ClearAllState,
            GetStateKeys,
            GetPromise,
            PeekPromise,
            CompletePromise,
            Sleep,
            Call,
            OneWayCall,
            Awakeable,
            CompleteAwakeable,
            Run
        })
    }

    fn read_entry_name(
        entry_type: EntryType,
        entry_value: Bytes,
    ) -> Result<Option<String>, RawEntryCodecError> {
        Ok(NamedEntryTemplate::decode(entry_value)
            .map_err(|e| {
                RawEntryCodecError::new(entry_type, ErrorKind::Decode {
                    source: Some(e.into()),
                })
            })?
            .name)
    }

    fn write_completion<InvokeEnrichmentResult: Debug, AwakeableEnrichmentResult: Debug>(
        entry: &mut RawEntry<InvokeEnrichmentResult, AwakeableEnrichmentResult>,
        completion_result: CompletionResult,
    ) -> Result<(), RawEntryCodecError> {
        debug_assert_eq!(
            entry.header().is_completed(),
            Some(false),
            "Entry '{:?}' is already completed",
            entry
        );

        // Prepare the result to serialize in protobuf
        let completion_result_message = match completion_result {
            CompletionResult::Empty => {
                service_protocol::completion_message::Result::Empty(service_protocol::Empty {})
            }
            CompletionResult::Success(b) => service_protocol::completion_message::Result::Value(b),
            CompletionResult::Failure(code, message) => {
                service_protocol::completion_message::Result::Failure(service_protocol::Failure {
                    code: code.into(),
                    message: message.to_string(),
                })
            }
        };

        // Prepare a buffer for the result
        // TODO perhaps use SegmentedBuf here to avoid allocating?
        let len = entry.serialized_entry().len() + completion_result_message.encoded_len();
        let mut result_buf = BytesMut::with_capacity(len);

        // Concatenate entry + result
        // The reason why encoding completion_message_result works is that by convention the tags
        // of completion message are the same used by completable entries.
        // See the service_protocol protobuf definition for more details.
        // https://protobuf.dev/programming-guides/encoding/#last-one-wins
        result_buf.put(mem::take(entry.serialized_entry_mut()));
        completion_result_message.encode(&mut result_buf);

        // Write back to the entry the new buffer and the completed flag
        *entry.serialized_entry_mut() = result_buf.freeze();
        entry.header_mut().mark_completed();

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use bytes::Bytes;
    use restate_sdk_types::{
        journal::{
            raw::{PlainEntryHeader, PlainRawEntry},
            CompletionResult, Entry, EntryResult,
        },
        service_protocol,
    };

    #[test]
    fn complete_invoke() {
        let invoke_result = Bytes::from_static(b"output");

        // Create an invoke entry
        let raw_entry: PlainRawEntry = RawEntry::new(
            PlainEntryHeader::Call {
                is_completed: false,
                enrichment_result: None,
            },
            service_protocol::CallEntryMessage {
                service_name: "MySvc".to_string(),
                handler_name: "MyMethod".to_string(),

                parameter: Bytes::from_static(b"input"),
                ..service_protocol::CallEntryMessage::default()
            }
            .encode_to_vec()
            .into(),
        );

        // Complete the expected entry directly on the materialized model
        let mut expected_entry = raw_entry
            .deserialize_entry_ref::<ProtobufRawEntryCodec>()
            .unwrap();
        match &mut expected_entry {
            Entry::Call(invoke_entry_inner) => {
                invoke_entry_inner.result = Some(EntryResult::Success(invoke_result.clone()))
            }
            _ => unreachable!(),
        };

        // Complete the raw entry
        let mut actual_raw_entry = raw_entry;
        ProtobufRawEntryCodec::write_completion(
            &mut actual_raw_entry,
            CompletionResult::Success(invoke_result),
        )
        .unwrap();
        let actual_entry = actual_raw_entry
            .deserialize_entry_ref::<ProtobufRawEntryCodec>()
            .unwrap();

        assert_eq!(actual_raw_entry.header().is_completed(), Some(true));
        assert_eq!(actual_entry, expected_entry);
    }
}
