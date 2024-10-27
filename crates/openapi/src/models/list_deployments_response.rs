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
pub struct ListDeploymentsResponse {
    #[serde(rename = "deployments")]
    pub deployments: Vec<models::DeploymentResponse>,
}

impl ListDeploymentsResponse {
    pub fn new(deployments: Vec<models::DeploymentResponse>) -> ListDeploymentsResponse {
        ListDeploymentsResponse {
            deployments,
        }
    }
}

