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
pub enum DeletionMode {
    #[serde(rename = "Cancel")]
    Cancel,
    #[serde(rename = "Kill")]
    Kill,
    #[serde(rename = "Purge")]
    Purge,

}

impl std::fmt::Display for DeletionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Kill => write!(f, "Kill"),
            Self::Purge => write!(f, "Purge"),
        }
    }
}

impl Default for DeletionMode {
    fn default() -> DeletionMode {
        Self::Cancel
    }
}

