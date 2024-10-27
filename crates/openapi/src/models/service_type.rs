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
pub enum ServiceType {
    #[serde(rename = "Service")]
    Service,
    #[serde(rename = "VirtualObject")]
    VirtualObject,
    #[serde(rename = "Workflow")]
    Workflow,

}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Service => write!(f, "Service"),
            Self::VirtualObject => write!(f, "VirtualObject"),
            Self::Workflow => write!(f, "Workflow"),
        }
    }
}

impl Default for ServiceType {
    fn default() -> ServiceType {
        Self::Service
    }
}

