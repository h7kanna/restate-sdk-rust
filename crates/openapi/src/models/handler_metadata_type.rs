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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum HandlerMetadataType {
    #[serde(rename = "Exclusive")]
    Exclusive,
    #[serde(rename = "Shared")]
    Shared,
    #[serde(rename = "Workflow")]
    Workflow,

}

impl std::fmt::Display for HandlerMetadataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Exclusive => write!(f, "Exclusive"),
            Self::Shared => write!(f, "Shared"),
            Self::Workflow => write!(f, "Workflow"),
        }
    }
}

impl Default for HandlerMetadataType {
    fn default() -> HandlerMetadataType {
        Self::Exclusive
    }
}

