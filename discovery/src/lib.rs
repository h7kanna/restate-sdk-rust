use serde::{Deserialize, Serialize};
/// Error types.
pub mod error {
    /// Error from a TryFrom or FromStr implementation.
    pub struct ConversionError(std::borrow::Cow<'static, str>);
    impl std::error::Error for ConversionError {}
    impl std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
///Restate endpoint manifest v1
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "$id": "https://restate.dev/endpoint.manifest.json",
///  "title": "Endpoint",
///  "description": "Restate endpoint manifest v1",
///  "type": "object",
///  "required": [
///    "maxProtocolVersion",
///    "minProtocolVersion",
///    "services"
///  ],
///  "properties": {
///    "maxProtocolVersion": {
///      "description": "Maximum supported protocol version",
///      "type": "integer",
///      "maximum": 2147483647.0,
///      "minimum": 1.0
///    },
///    "minProtocolVersion": {
///      "description": "Minimum supported protocol version",
///      "type": "integer",
///      "maximum": 2147483647.0,
///      "minimum": 1.0
///    },
///    "protocolMode": {
///      "title": "ProtocolMode",
///      "enum": [
///        "BIDI_STREAM",
///        "REQUEST_RESPONSE"
///      ]
///    },
///    "services": {
///      "type": "array",
///      "items": {
///        "title": "Service",
///        "type": "object",
///        "required": [
///          "handlers",
///          "name",
///          "ty"
///        ],
///        "properties": {
///          "handlers": {
///            "type": "array",
///            "items": {
///              "title": "Handler",
///              "type": "object",
///              "required": [
///                "name"
///              ],
///              "properties": {
///                "input": {
///                  "title": "InputPayload",
///                  "description": "Description of an input payload. This will be used by Restate to validate incoming requests.",
///                  "type": "object",
///                  "properties": {
///                    "contentType": {
///                      "description": "Content type of the input. It can accept wildcards, in the same format as the 'Accept' header. When this field is unset, it implies emptiness, meaning no content-type/body is expected.",
///                      "type": "string"
///                    },
///                    "jsonSchema": {},
///                    "required": {
///                      "description": "If true, a body MUST be sent with a content-type, even if the body length is zero.",
///                      "type": "boolean"
///                    }
///                  },
///                  "additionalProperties": false
///                },
///                "name": {
///                  "type": "string",
///                  "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$"
///                },
///                "output": {
///                  "title": "OutputPayload",
///                  "description": "Description of an output payload.",
///                  "type": "object",
///                  "properties": {
///                    "contentType": {
///                      "description": "Content type set on output. This will be used by Restate to set the output content type at the ingress.",
///                      "type": "string"
///                    },
///                    "jsonSchema": {},
///                    "setContentTypeIfEmpty": {
///                      "description": "If true, the specified content-type is set even if the output is empty.",
///                      "type": "boolean"
///                    }
///                  },
///                  "additionalProperties": false
///                },
///                "ty": {
///                  "title": "HandlerType",
///                  "description": "If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.",
///                  "enum": [
///                    "WORKFLOW",
///                    "EXCLUSIVE",
///                    "SHARED"
///                  ]
///                }
///              },
///              "additionalProperties": false
///            }
///          },
///          "name": {
///            "type": "string",
///            "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9._-]*$"
///          },
///          "ty": {
///            "title": "ServiceType",
///            "enum": [
///              "VIRTUAL_OBJECT",
///              "SERVICE",
///              "WORKFLOW"
///            ]
///          }
///        },
///        "additionalProperties": false
///      }
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    ///Maximum supported protocol version
    #[serde(rename = "maxProtocolVersion")]
    pub max_protocol_version: i64,
    ///Minimum supported protocol version
    #[serde(rename = "minProtocolVersion")]
    pub min_protocol_version: i64,
    #[serde(rename = "protocolMode", default, skip_serializing_if = "Option::is_none")]
    pub protocol_mode: Option<ProtocolMode>,
    pub services: Vec<Service>,
}
impl From<&Endpoint> for Endpoint {
    fn from(value: &Endpoint) -> Self {
        value.clone()
    }
}
///Handler
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "Handler",
///  "type": "object",
///  "required": [
///    "name"
///  ],
///  "properties": {
///    "input": {
///      "title": "InputPayload",
///      "description": "Description of an input payload. This will be used by Restate to validate incoming requests.",
///      "type": "object",
///      "properties": {
///        "contentType": {
///          "description": "Content type of the input. It can accept wildcards, in the same format as the 'Accept' header. When this field is unset, it implies emptiness, meaning no content-type/body is expected.",
///          "type": "string"
///        },
///        "jsonSchema": {},
///        "required": {
///          "description": "If true, a body MUST be sent with a content-type, even if the body length is zero.",
///          "type": "boolean"
///        }
///      },
///      "additionalProperties": false
///    },
///    "name": {
///      "type": "string",
///      "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$"
///    },
///    "output": {
///      "title": "OutputPayload",
///      "description": "Description of an output payload.",
///      "type": "object",
///      "properties": {
///        "contentType": {
///          "description": "Content type set on output. This will be used by Restate to set the output content type at the ingress.",
///          "type": "string"
///        },
///        "jsonSchema": {},
///        "setContentTypeIfEmpty": {
///          "description": "If true, the specified content-type is set even if the output is empty.",
///          "type": "boolean"
///        }
///      },
///      "additionalProperties": false
///    },
///    "ty": {
///      "title": "HandlerType",
///      "description": "If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.",
///      "enum": [
///        "WORKFLOW",
///        "EXCLUSIVE",
///        "SHARED"
///      ]
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Handler {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<InputPayload>,
    pub name: HandlerName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputPayload>,
    ///If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<HandlerType>,
}
impl From<&Handler> for Handler {
    fn from(value: &Handler) -> Self {
        value.clone()
    }
}
///HandlerName
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "string",
///  "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct HandlerName(String);
impl std::ops::Deref for HandlerName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<HandlerName> for String {
    fn from(value: HandlerName) -> Self {
        value.0
    }
}
impl From<&HandlerName> for HandlerName {
    fn from(value: &HandlerName) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for HandlerName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$\"".into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for HandlerName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for HandlerName {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for HandlerName {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for HandlerName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "HandlerType",
///  "description": "If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.",
///  "enum": [
///    "WORKFLOW",
///    "EXCLUSIVE",
///    "SHARED"
///  ]
///}
/// ```
/// </details>
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize
)]
pub enum HandlerType {
    #[serde(rename = "WORKFLOW")]
    Workflow,
    #[serde(rename = "EXCLUSIVE")]
    Exclusive,
    #[serde(rename = "SHARED")]
    Shared,
}
impl From<&HandlerType> for HandlerType {
    fn from(value: &HandlerType) -> Self {
        value.clone()
    }
}
impl ToString for HandlerType {
    fn to_string(&self) -> String {
        match *self {
            Self::Workflow => "WORKFLOW".to_string(),
            Self::Exclusive => "EXCLUSIVE".to_string(),
            Self::Shared => "SHARED".to_string(),
        }
    }
}
impl std::str::FromStr for HandlerType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        match value {
            "WORKFLOW" => Ok(Self::Workflow),
            "EXCLUSIVE" => Ok(Self::Exclusive),
            "SHARED" => Ok(Self::Shared),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for HandlerType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for HandlerType {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for HandlerType {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
///Description of an input payload. This will be used by Restate to validate incoming requests.
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "InputPayload",
///  "description": "Description of an input payload. This will be used by Restate to validate incoming requests.",
///  "type": "object",
///  "properties": {
///    "contentType": {
///      "description": "Content type of the input. It can accept wildcards, in the same format as the 'Accept' header. When this field is unset, it implies emptiness, meaning no content-type/body is expected.",
///      "type": "string"
///    },
///    "jsonSchema": {},
///    "required": {
///      "description": "If true, a body MUST be sent with a content-type, even if the body length is zero.",
///      "type": "boolean"
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputPayload {
    ///Content type of the input. It can accept wildcards, in the same format as the 'Accept' header. When this field is unset, it implies emptiness, meaning no content-type/body is expected.
    #[serde(rename = "contentType", default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "jsonSchema", default, skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    ///If true, a body MUST be sent with a content-type, even if the body length is zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}
impl From<&InputPayload> for InputPayload {
    fn from(value: &InputPayload) -> Self {
        value.clone()
    }
}
///Description of an output payload.
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "OutputPayload",
///  "description": "Description of an output payload.",
///  "type": "object",
///  "properties": {
///    "contentType": {
///      "description": "Content type set on output. This will be used by Restate to set the output content type at the ingress.",
///      "type": "string"
///    },
///    "jsonSchema": {},
///    "setContentTypeIfEmpty": {
///      "description": "If true, the specified content-type is set even if the output is empty.",
///      "type": "boolean"
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputPayload {
    ///Content type set on output. This will be used by Restate to set the output content type at the ingress.
    #[serde(rename = "contentType", default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "jsonSchema", default, skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    ///If true, the specified content-type is set even if the output is empty.
    #[serde(
        rename = "setContentTypeIfEmpty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub set_content_type_if_empty: Option<bool>,
}
impl From<&OutputPayload> for OutputPayload {
    fn from(value: &OutputPayload) -> Self {
        value.clone()
    }
}
///ProtocolMode
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "ProtocolMode",
///  "enum": [
///    "BIDI_STREAM",
///    "REQUEST_RESPONSE"
///  ]
///}
/// ```
/// </details>
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize
)]
pub enum ProtocolMode {
    #[serde(rename = "BIDI_STREAM")]
    BidiStream,
    #[serde(rename = "REQUEST_RESPONSE")]
    RequestResponse,
}
impl From<&ProtocolMode> for ProtocolMode {
    fn from(value: &ProtocolMode) -> Self {
        value.clone()
    }
}
impl ToString for ProtocolMode {
    fn to_string(&self) -> String {
        match *self {
            Self::BidiStream => "BIDI_STREAM".to_string(),
            Self::RequestResponse => "REQUEST_RESPONSE".to_string(),
        }
    }
}
impl std::str::FromStr for ProtocolMode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        match value {
            "BIDI_STREAM" => Ok(Self::BidiStream),
            "REQUEST_RESPONSE" => Ok(Self::RequestResponse),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for ProtocolMode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ProtocolMode {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ProtocolMode {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
///Service
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "Service",
///  "type": "object",
///  "required": [
///    "handlers",
///    "name",
///    "ty"
///  ],
///  "properties": {
///    "handlers": {
///      "type": "array",
///      "items": {
///        "title": "Handler",
///        "type": "object",
///        "required": [
///          "name"
///        ],
///        "properties": {
///          "input": {
///            "title": "InputPayload",
///            "description": "Description of an input payload. This will be used by Restate to validate incoming requests.",
///            "type": "object",
///            "properties": {
///              "contentType": {
///                "description": "Content type of the input. It can accept wildcards, in the same format as the 'Accept' header. When this field is unset, it implies emptiness, meaning no content-type/body is expected.",
///                "type": "string"
///              },
///              "jsonSchema": {},
///              "required": {
///                "description": "If true, a body MUST be sent with a content-type, even if the body length is zero.",
///                "type": "boolean"
///              }
///            },
///            "additionalProperties": false
///          },
///          "name": {
///            "type": "string",
///            "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9_]*$"
///          },
///          "output": {
///            "title": "OutputPayload",
///            "description": "Description of an output payload.",
///            "type": "object",
///            "properties": {
///              "contentType": {
///                "description": "Content type set on output. This will be used by Restate to set the output content type at the ingress.",
///                "type": "string"
///              },
///              "jsonSchema": {},
///              "setContentTypeIfEmpty": {
///                "description": "If true, the specified content-type is set even if the output is empty.",
///                "type": "boolean"
///              }
///            },
///            "additionalProperties": false
///          },
///          "ty": {
///            "title": "HandlerType",
///            "description": "If unspecified, defaults to EXCLUSIVE for Virtual Object or WORKFLOW for Workflows. This should be unset for Services.",
///            "enum": [
///              "WORKFLOW",
///              "EXCLUSIVE",
///              "SHARED"
///            ]
///          }
///        },
///        "additionalProperties": false
///      }
///    },
///    "name": {
///      "type": "string",
///      "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9._-]*$"
///    },
///    "ty": {
///      "title": "ServiceType",
///      "enum": [
///        "VIRTUAL_OBJECT",
///        "SERVICE",
///        "WORKFLOW"
///      ]
///    }
///  },
///  "additionalProperties": false
///}
/// ```
/// </details>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub handlers: Vec<Handler>,
    pub name: ServiceName,
    pub ty: ServiceType,
}
impl From<&Service> for Service {
    fn from(value: &Service) -> Self {
        value.clone()
    }
}
///ServiceName
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "type": "string",
///  "pattern": "^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9._-]*$"
///}
/// ```
/// </details>
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ServiceName(String);
impl std::ops::Deref for ServiceName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<ServiceName> for String {
    fn from(value: ServiceName) -> Self {
        value.0
    }
}
impl From<&ServiceName> for ServiceName {
    fn from(value: &ServiceName) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for ServiceName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if regress::Regex::new("^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9._-]*$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err(
                "doesn't match pattern \"^([a-zA-Z]|_[a-zA-Z0-9])[a-zA-Z0-9._-]*$\""
                    .into(),
            );
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for ServiceName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ServiceName {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ServiceName {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for ServiceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
///ServiceType
///
/// <details><summary>JSON schema</summary>
///
/// ```json
///{
///  "title": "ServiceType",
///  "enum": [
///    "VIRTUAL_OBJECT",
///    "SERVICE",
///    "WORKFLOW"
///  ]
///}
/// ```
/// </details>
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize
)]
pub enum ServiceType {
    #[serde(rename = "VIRTUAL_OBJECT")]
    VirtualObject,
    #[serde(rename = "SERVICE")]
    Service,
    #[serde(rename = "WORKFLOW")]
    Workflow,
}
impl From<&ServiceType> for ServiceType {
    fn from(value: &ServiceType) -> Self {
        value.clone()
    }
}
impl ToString for ServiceType {
    fn to_string(&self) -> String {
        match *self {
            Self::VirtualObject => "VIRTUAL_OBJECT".to_string(),
            Self::Service => "SERVICE".to_string(),
            Self::Workflow => "WORKFLOW".to_string(),
        }
    }
}
impl std::str::FromStr for ServiceType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        match value {
            "VIRTUAL_OBJECT" => Ok(Self::VirtualObject),
            "SERVICE" => Ok(Self::Service),
            "WORKFLOW" => Ok(Self::Workflow),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for ServiceType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ServiceType {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ServiceType {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
