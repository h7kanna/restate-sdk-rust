/*
 * Admin API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.2
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModifyServiceRequest {
    /// If true, the service can be invoked through the ingress. If false, the service can be invoked only from another Restate service.
    #[serde(rename = "public", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub public: Option<Option<bool>>,
    /// Modify the retention of idempotent requests for this service.  Can be configured using the [`humantime`](https://docs.rs/humantime/latest/humantime/fn.parse_duration.html) format or the ISO8601.
    #[serde(rename = "idempotency_retention", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub idempotency_retention: Option<Option<String>>,
    /// Modify the retention of the workflow completion. This can be modified only for workflow services!  Can be configured using the [`humantime`](https://docs.rs/humantime/latest/humantime/fn.parse_duration.html) format or the ISO8601.
    #[serde(rename = "workflow_completion_retention", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub workflow_completion_retention: Option<Option<String>>,
}

impl ModifyServiceRequest {
    pub fn new() -> ModifyServiceRequest {
        ModifyServiceRequest {
            public: None,
            idempotency_retention: None,
            workflow_completion_retention: None,
        }
    }
}

