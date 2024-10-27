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

/// ErrorDescriptionResponse : Error details of the response
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ErrorDescriptionResponse {
    #[serde(rename = "message")]
    pub message: String,
    /// Restate error code describing this error
    #[serde(rename = "restate_code", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub restate_code: Option<Option<String>>,
}

impl ErrorDescriptionResponse {
    /// Error details of the response
    pub fn new(message: String) -> ErrorDescriptionResponse {
        ErrorDescriptionResponse {
            message,
            restate_code: None,
        }
    }
}

