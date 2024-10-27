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
pub struct DeploymentResponseAnyOf {
    #[serde(rename = "uri")]
    pub uri: String,
    #[serde(rename = "protocol_type")]
    pub protocol_type: models::ProtocolType,
    #[serde(rename = "http_version")]
    pub http_version: String,
    #[serde(rename = "additional_headers", skip_serializing_if = "Option::is_none")]
    pub additional_headers: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "min_protocol_version")]
    pub min_protocol_version: i32,
    #[serde(rename = "max_protocol_version")]
    pub max_protocol_version: i32,
}

impl DeploymentResponseAnyOf {
    pub fn new(uri: String, protocol_type: models::ProtocolType, http_version: String, created_at: String, min_protocol_version: i32, max_protocol_version: i32) -> DeploymentResponseAnyOf {
        DeploymentResponseAnyOf {
            uri,
            protocol_type,
            http_version,
            additional_headers: None,
            created_at,
            min_protocol_version,
            max_protocol_version,
        }
    }
}

