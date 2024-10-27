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
pub struct RegisterDeploymentResponse {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "services")]
    pub services: Vec<models::ServiceMetadata>,
}

impl RegisterDeploymentResponse {
    pub fn new(id: String, services: Vec<models::ServiceMetadata>) -> RegisterDeploymentResponse {
        RegisterDeploymentResponse {
            id,
            services,
        }
    }
}

