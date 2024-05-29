/*
 * Admin API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.9.1
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetailedDeploymentResponse {
    #[serde(rename = "id")]
    pub id: String,
    /// List of services exposed by this deployment.
    #[serde(rename = "services")]
    pub services: Vec<models::ServiceMetadata>,
    #[serde(rename = "uri")]
    pub uri: String,
    #[serde(rename = "protocol_type")]
    pub protocol_type: models::ProtocolType,
    #[serde(rename = "additional_headers", skip_serializing_if = "Option::is_none")]
    pub additional_headers: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "min_protocol_version")]
    pub min_protocol_version: i32,
    #[serde(rename = "max_protocol_version")]
    pub max_protocol_version: i32,
    #[serde(rename = "arn")]
    pub arn: String,
    #[serde(rename = "assume_role_arn", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub assume_role_arn: Option<Option<String>>,
}

impl DetailedDeploymentResponse {
    pub fn new(id: String, services: Vec<models::ServiceMetadata>, uri: String, protocol_type: models::ProtocolType, created_at: String, min_protocol_version: i32, max_protocol_version: i32, arn: String) -> DetailedDeploymentResponse {
        DetailedDeploymentResponse {
            id,
            services,
            uri,
            protocol_type,
            additional_headers: None,
            created_at,
            min_protocol_version,
            max_protocol_version,
            arn,
            assume_role_arn: None,
        }
    }
}

