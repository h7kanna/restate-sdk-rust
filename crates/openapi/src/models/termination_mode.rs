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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TerminationMode {
    #[serde(rename = "Cancel")]
    Cancel,
    #[serde(rename = "Kill")]
    Kill,

}

impl std::fmt::Display for TerminationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Kill => write!(f, "Kill"),
        }
    }
}

impl Default for TerminationMode {
    fn default() -> TerminationMode {
        Self::Cancel
    }
}

