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
pub struct ServiceNameRevPair {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "revision")]
    pub revision: i32,
}

impl ServiceNameRevPair {
    pub fn new(name: String, revision: i32) -> ServiceNameRevPair {
        ServiceNameRevPair {
            name,
            revision,
        }
    }
}

