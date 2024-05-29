//! Restate Rust SDK Test Utils

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
use bytes::Buf;
use chrono::{DateTime, Local, TimeZone};
use reqwest::Method;
use restate_sdk_types::journal::raw::{PlainEntryHeader, PlainRawEntry};
use restate_service_protocol::message::ProtocolMessage;
use serde::Serialize;
use std::{fmt::Display, time::Duration};
use thiserror::Error;

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
pub struct SqlQueryRequest {
    pub query: String,
}

pub struct SqlResponse {
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
    raw: Option<Vec<u8>>,
}

pub struct TestRestateServer {}

pub struct TestDriver {}

impl TestRestateServer {
    async fn run_query(&self, invocation_id: String) -> Result<SqlResponse, Error> {
        let raw_client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{} {}-{}",
                env!("CARGO_PKG_NAME"),
                "0.1.0",
                std::env::consts::OS,
                std::env::consts::ARCH,
            ))
            .connect_timeout(Duration::from_secs(30))
            .build()?;

        let client = raw_client
            .request(Method::POST, "http://localhost:9070/query")
            .timeout(Duration::from_secs(30));

        let query = format!(
            "SELECT
            sj.index,
            sj.entry_type,
            sj.completed,
            sj.raw
        FROM sys_journal sj
        WHERE
            sj.id = '{}'
        ORDER BY index DESC
        LIMIT {}",
            invocation_id, 100,
        );

        let response = client.json(&SqlQueryRequest { query }).send().await?;

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

    pub async fn query_journal(&self, invocation_id: String) -> Vec<PlainRawEntry> {
        let mut journal = self
            .run_query_and_map_results::<JournalRowResult>(invocation_id)
            .await
            .unwrap()
            .map(|row| {
                let index = row.index.expect("index");
                let is_completed = row.completed.unwrap_or_default();
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
                PlainRawEntry::new(header, row.raw.unwrap().into())
            })
            .collect::<Vec<_>>();
        journal.reverse();
        journal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::{fs, path::Path};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_query() {
        let invocation_id = "inv_1llEAZMZzxgv6Sz2ZUQBSJmNCSnaqnVXQl";
        let output_file = false;
        let test_server = TestRestateServer {};
        let journal = test_server.query_journal(invocation_id.to_owned()).await;
        println!("{:?}", journal);
        let json = serde_json::to_string(&journal).unwrap();

        if output_file {
            let mut out_file = Path::new(".").to_path_buf();
            out_file.push("history.json");
            fs::write(out_file, json).unwrap();
        }

        let journal = journal
            .into_iter()
            .map(|entry| ProtocolMessage::UnparsedEntry(entry))
            .collect::<Vec<_>>();
        let start_message = ProtocolMessage::new_start_message(
            Bytes::from(invocation_id),
            invocation_id.to_string(),
            None,
            journal.len() as u32,
            false,
            vec![],
        );
        println!("{:?}", start_message);
    }
}
