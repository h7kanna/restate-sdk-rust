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
use restate_sdk_types::journal::EntryType;
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
    raw: Option<Vec<u8>>,
}

pub struct TestRestateServer {}

pub struct TestDriver {}

impl TestRestateServer {
    pub async fn run_query(&self, invocation_id: String) -> Result<SqlResponse, Error> {
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
            "SELECT * FROM sys_journal sj WHERE sj.id = '{}' ORDER BY index LIMIT 1000",
            invocation_id
        );

        let query = format!(
            "SELECT
            sj.index,
            sj.entry_type,
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

    pub async fn run_query_and_map_results<T: ArrowDeserialize + ArrowField<Type = T> + 'static>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use restate_sdk_types::journal::{raw::RawEntryCodec, Entry};
    use restate_service_protocol::codec::ProtobufRawEntryCodec;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_query() {
        let invocation_id = "inv_13F4CrYOOGbN1HKesxBpxbiTJVbb46eQ25".to_string();
        let test_server = TestRestateServer {};

        let mut journal: Vec<Entry> = test_server
            .run_query_and_map_results::<JournalRowResult>(invocation_id)
            .await
            .unwrap()
            .map(|row| {
                let index = row.index.expect("index");
                let entry_type = match row.entry_type.expect("entry_type").as_str() {
                    "Input" => EntryType::Input,
                    "Output" => EntryType::Output,
                    "GetState" => EntryType::GetState,
                    "SetState" => EntryType::SetState,
                    "ClearState" => EntryType::ClearState,
                    "GetStateKeys" => EntryType::GetStateKeys,
                    "ClearAllState" => EntryType::ClearAllState,
                    "GetPromise" => EntryType::GetPromise,
                    "PeekPromise" => EntryType::PeekPromise,
                    "CompletePromise" => EntryType::CompletePromise,
                    "Sleep" => EntryType::Sleep,
                    "Call" => EntryType::Call,
                    "OneWayCall" => EntryType::OneWayCall,
                    "Awakeable" => EntryType::Awakeable,
                    "Run" => EntryType::Run,
                    t => EntryType::Custom,
                };
                ProtobufRawEntryCodec::deserialize(entry_type.clone(), row.raw.unwrap().into()).unwrap()
            })
            .collect();

        // Sort by seq.
        journal.reverse();

        println!("{:?}", journal);
    }
}
