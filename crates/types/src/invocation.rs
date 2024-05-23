use std::fmt;
use bytestring::ByteString;
use prost::bytes::Bytes;
use crate::errors::InvocationError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: ByteString,
    pub value: ByteString,
}

impl Header {
    pub fn new(name: impl Into<ByteString>, value: impl Into<ByteString>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseResult {
    Success(Bytes),
    Failure(InvocationError),
}

impl From<Result<Bytes, InvocationError>> for ResponseResult {
    fn from(value: Result<Bytes, InvocationError>) -> Self {
        match value {
            Ok(v) => ResponseResult::Success(v),
            Err(e) => ResponseResult::Failure(e),
        }
    }
}

impl From<ResponseResult> for Result<Bytes, InvocationError> {
    fn from(value: ResponseResult) -> Self {
        match value {
            ResponseResult::Success(bytes) => Ok(bytes),
            ResponseResult::Failure(e) => Err(e),
        }
    }
}

impl From<InvocationError> for ResponseResult {
    fn from(e: InvocationError) -> Self {
        ResponseResult::Failure(e)
    }
}

impl From<&InvocationError> for ResponseResult {
    fn from(e: &InvocationError) -> Self {
        ResponseResult::Failure(e.clone())
    }
}