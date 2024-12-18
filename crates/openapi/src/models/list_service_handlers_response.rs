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
pub struct ListServiceHandlersResponse {
    #[serde(rename = "handlers")]
    pub handlers: Vec<models::HandlerMetadata>,
}

impl ListServiceHandlersResponse {
    pub fn new(handlers: Vec<models::HandlerMetadata>) -> ListServiceHandlersResponse {
        ListServiceHandlersResponse {
            handlers,
        }
    }
}

