//! Restate Rust SDK Utils

use arrow::{
    array::{AsArray, StructArray},
    datatypes::{ArrowPrimitiveType, Date64Type, SchemaRef},
    error::ArrowError,
    ipc::reader::StreamReader,
    record_batch::RecordBatch,
};
use arrow_convert::{
    deserialize::{arrow_array_deserialize_iterator, ArrowDeserialize},
    field::ArrowField,
    ArrowDeserialize, ArrowField,
};
use async_stream::try_stream;
use bytes::{Buf, Bytes};
use chrono::{DateTime, Local, TimeZone};
use futures_util::Stream;
use reqwest::{Client, Method, RequestBuilder};
pub use restate_sdk_types::journal::{
    raw::{PlainEntryHeader, PlainRawEntry},
    EntryType,
};
pub use restate_service_protocol::message::{MessageType, ProtocolMessage};
use serde::Serialize;
use std::{
    collections::VecDeque,
    fmt::Display,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use thiserror::Error;
use tracing::{debug, field::debug};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    #[error("(Protocol error) {0}")]
    Serialization(#[from] serde_json::Error),
    Network(#[from] reqwest::Error),
    Arrow(#[from] ArrowError),
    #[error("Mapping from query '{0}': {1}")]
    Mapping(String, #[source] ArrowError),
    UrlParse(#[from] url::ParseError),
}

#[derive(Serialize, Debug, Clone)]
struct SqlQueryRequest {
    pub query: String,
}

struct SqlResponse {
    pub schema: SchemaRef,
    pub batches: Vec<RecordBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RestateDateTime(DateTime<Local>);

impl From<RestateDateTime> for DateTime<Local> {
    fn from(value: RestateDateTime) -> Self {
        value.0
    }
}

impl arrow_convert::field::ArrowField for RestateDateTime {
    type Type = Self;

    #[inline]
    fn data_type() -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Date64
    }
}

impl arrow_convert::deserialize::ArrowDeserialize for RestateDateTime {
    type ArrayType = arrow::array::Date64Array;

    #[inline]
    fn arrow_deserialize(v: Option<i64>) -> Option<Self> {
        v.and_then(arrow::temporal_conversions::as_datetime::<Date64Type>)
            .map(|naive| Local.from_utc_datetime(&naive))
            .map(RestateDateTime)
    }
}

// enable Vec<RestateDateTime>
arrow_convert::arrow_enable_vec_for_type!(RestateDateTime);

#[derive(Debug, Clone, PartialEq, ArrowField, ArrowDeserialize)]
struct JournalRowResult {
    index: Option<u32>,
    entry_type: Option<String>,
    completed: Option<bool>,
    name: Option<String>,
    raw: Option<Vec<u8>>,
}

pub type HistoryMessage = (Option<String>, MessageType, ProtocolMessage);

pub type History = VecDeque<HistoryMessage>;

pub struct JournalClient {
    raw_client: Client,
    request_builder: RequestBuilder,
}

pub struct JournalWatchOptions {
    pub interval_millis: u64,
    pub cancellation_token: Arc<AtomicBool>,
}

impl JournalClient {
    pub async fn new(admin_url: String) -> Result<Self, Error> {
        let raw_client = Client::builder()
            .user_agent(format!(
                "{}/{} {}-{}",
                env!("CARGO_PKG_NAME"),
                "0.1.0",
                std::env::consts::OS,
                std::env::consts::ARCH,
            ))
            .connect_timeout(Duration::from_secs(30))
            .build()?;

        let request_builder = raw_client
            .request(Method::POST, format!("{}/query", admin_url))
            .timeout(Duration::from_secs(30));

        Ok(Self {
            raw_client,
            request_builder,
        })
    }

    async fn run_query(&self, invocation_id: String) -> Result<SqlResponse, Error> {
        let query = format!(
            "SELECT
            sj.index,
            sj.entry_type,
            sj.completed,
            sj.name,
            sj.raw
        FROM sys_journal sj
        WHERE
            sj.id = '{}'
        ORDER BY index DESC
        LIMIT {}",
            invocation_id, 100,
        );

        let response = self
            .request_builder
            .try_clone()
            .unwrap()
            .json(&SqlQueryRequest { query })
            .send()
            .await?;

        let payload = response.bytes().await?.reader();
        let reader = StreamReader::try_new(payload, None)?;
        let schema = reader.schema();

        let mut batches = Vec::new();
        for batch in reader {
            batches.push(batch?);
        }

        Ok(SqlResponse { schema, batches })
    }

    async fn run_query_and_map_results<T: ArrowDeserialize + ArrowField<Type = T> + 'static>(
        &self,
        invocation_id: String,
    ) -> Result<impl Iterator<Item = T>, Error> {
        let sql_response = self.run_query(invocation_id.clone()).await?;
        let mut results = Vec::new();
        for batch in sql_response.batches {
            let n = batch.num_rows();
            if n == 0 {
                continue;
            }
            results.reserve(n);

            // Map results using arrow_convert
            for row in arrow_array_deserialize_iterator::<T>(&StructArray::from(batch))
                .map_err(|e| Error::Mapping(invocation_id.clone(), e))?
            {
                results.push(row);
            }
        }
        Ok(results.into_iter())
    }

    pub async fn query_journal(&self, invocation_id: String) -> Vec<(Option<String>, PlainRawEntry)> {
        let mut journal = self
            .run_query_and_map_results::<JournalRowResult>(invocation_id)
            .await
            .unwrap()
            .map(|row| {
                let index = row.index.expect("index");
                let is_completed = row.completed.unwrap_or_default();
                let name = row.name;
                let header = match row.entry_type.expect("entry_type").as_str() {
                    "Input" => PlainEntryHeader::Input,
                    "Output" => PlainEntryHeader::Output,
                    "GetState" => PlainEntryHeader::GetState { is_completed },
                    "SetState" => PlainEntryHeader::SetState,
                    "ClearState" => PlainEntryHeader::ClearState,
                    "GetStateKeys" => PlainEntryHeader::GetStateKeys { is_completed },
                    "ClearAllState" => PlainEntryHeader::ClearAllState,
                    "GetPromise" => PlainEntryHeader::GetPromise { is_completed },
                    "PeekPromise" => PlainEntryHeader::PeekPromise { is_completed },
                    "CompletePromise" => PlainEntryHeader::CompletePromise { is_completed },
                    "Sleep" => PlainEntryHeader::Sleep { is_completed },
                    "Call" => PlainEntryHeader::Call {
                        is_completed,
                        enrichment_result: None,
                    },
                    "OneWayCall" => PlainEntryHeader::OneWayCall {
                        enrichment_result: (),
                    },
                    "Awakeable" => PlainEntryHeader::Awakeable { is_completed },
                    "CompleteAwakeable" => PlainEntryHeader::CompleteAwakeable {
                        enrichment_result: (),
                    },
                    "Run" => PlainEntryHeader::Run,
                    t => PlainEntryHeader::Custom { code: 0 },
                };
                (name, PlainRawEntry::new(header, row.raw.unwrap().into()))
            })
            .collect::<Vec<_>>();
        journal.reverse();
        journal
    }

    pub async fn journal_to_protocol(&self, invocation_id: String) -> History {
        let journal = self.query_journal(invocation_id.clone()).await;
        let mut journal = journal
            .into_iter()
            .map(|message| {
                let entry = message.1;
                let message_type = match entry.header().as_entry_type() {
                    EntryType::Input => MessageType::InputEntry,
                    EntryType::Output => MessageType::OutputEntry,
                    EntryType::GetState => MessageType::GetStateEntry,
                    EntryType::SetState => MessageType::SetStateEntry,
                    EntryType::ClearState => MessageType::ClearStateEntry,
                    EntryType::GetStateKeys => MessageType::GetStateKeysEntry,
                    EntryType::ClearAllState => MessageType::ClearAllStateEntry,
                    EntryType::GetPromise => MessageType::GetPromiseEntry,
                    EntryType::PeekPromise => MessageType::PeekPromiseEntry,
                    EntryType::CompletePromise => MessageType::CompletePromiseEntry,
                    EntryType::Sleep => MessageType::SleepEntry,
                    EntryType::Call => MessageType::InvokeEntry,
                    EntryType::OneWayCall => MessageType::BackgroundInvokeEntry,
                    EntryType::Awakeable => MessageType::AwakeableEntry,
                    EntryType::CompleteAwakeable => MessageType::CompleteAwakeableEntry,
                    EntryType::Run => MessageType::SideEffectEntry,
                    EntryType::CancelInvocation => MessageType::CancelInvocationEntry,
                    EntryType::GetCallInvocationId => MessageType::GetCallInvocationIdEntry,
                    EntryType::Custom => MessageType::CustomEntry(0),
                };
                (message.0, message_type, ProtocolMessage::UnparsedEntry(entry))
            })
            .collect::<History>();
        let start_message = ProtocolMessage::new_start_message(
            Bytes::from(invocation_id.clone()),
            invocation_id,
            None,
            journal.len() as u32,
            false,
            vec![],
            0,
            Duration::ZERO,
        );
        journal.push_front((None, MessageType::Start, start_message));
        journal
    }

    pub async fn watch_journal(
        &self,
        invocation_id: String,
        options: JournalWatchOptions,
    ) -> impl Stream<Item = Result<HistoryMessage, Error>> + '_ {
        try_stream! {
            let mut existing = self.journal_to_protocol(invocation_id.clone()).await;
            for entry in existing.iter() {
                yield entry.clone();
            }
            debug!(
                "Journal polling started for Invocation: {}, History: {}",
                invocation_id,
                existing.len()
            );
            loop {
                if options.cancellation_token.load(Ordering::Relaxed) {
                    break;
                } else {
                    tokio::time::sleep(Duration::from_millis(options.interval_millis)).await;
                }

                let journal = self.journal_to_protocol(invocation_id.clone()).await;
                debug!(
                    "New entries received for Invocation: {}, History: {}",
                    invocation_id,
                    journal.len()
                );
                if existing.len() < journal.len() {
                    let start = journal.get(0).unwrap().clone();
                    yield start;
                    for entry in journal.iter().skip(existing.len()) {
                        let end = match entry.1 {
                            MessageType::Error | MessageType::End | MessageType::OutputEntry => true,
                            _ => false,
                        };
                        yield entry.clone();
                        if end {
                            break;
                        }
                    }
                    existing = journal;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{pin_mut, StreamExt};
    use restate_sdk_types::journal::{Entry, EntryResult};
    use restate_service_protocol::codec::ProtobufRawEntryCodec;
    use tracing::debug;
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_query() {
        let invocation_id = "inv_13WTOp5Lirno4meUO0SEIv3iJPn3uGUca5";
        let journal_client = JournalClient::new("http://localhost:9070".to_string())
            .await
            .unwrap();
        let cancellation_token = Arc::new(AtomicBool::new(false));
        let journal = journal_client
            .watch_journal(
                invocation_id.to_owned(),
                JournalWatchOptions {
                    interval_millis: 500,
                    cancellation_token: cancellation_token.clone(),
                },
            )
            .await;
        pin_mut!(journal);
        while let Some(message) = journal.next().await {
            match message {
                Ok(message) => match message.2 {
                    ProtocolMessage::UnparsedEntry(entry) => {
                        match entry.deserialize_entry::<ProtobufRawEntryCodec>().unwrap() {
                            Entry::Input(_) => {}
                            Entry::Output(_) => {}
                            Entry::GetState(_) => {}
                            Entry::SetState(_) => {}
                            Entry::ClearState(_) => {}
                            Entry::GetStateKeys(_) => {}
                            Entry::ClearAllState => {}
                            Entry::GetPromise(_) => {}
                            Entry::PeekPromise(_) => {}
                            Entry::CompletePromise(_) => {}
                            Entry::Sleep(_) => {}
                            Entry::Call(call) => match call.result {
                                Some(result) => match result {
                                    EntryResult::Success(bytes) => {
                                        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                                        debug!("Call: {:?}, Value: {}", message.0, json);
                                    }
                                    EntryResult::Failure(_, _) => {}
                                },
                                None => {}
                            },
                            Entry::OneWayCall(_) => {}
                            Entry::Awakeable(_) => {}
                            Entry::CompleteAwakeable(_) => {}
                            Entry::Run(_) => {}
                            Entry::CancelInvocation(_) => {}
                            Entry::GetCallInvocationId(_) => {}
                            Entry::Custom(_) => {}
                        }
                    }
                    _ => {}
                },
                Err(err) => {}
            }
        }
    }
}
