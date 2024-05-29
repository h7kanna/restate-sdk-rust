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
pub struct ListSubscriptionsResponse {
    #[serde(rename = "subscriptions")]
    pub subscriptions: Vec<models::SubscriptionResponse>,
}

impl ListSubscriptionsResponse {
    pub fn new(subscriptions: Vec<models::SubscriptionResponse>) -> ListSubscriptionsResponse {
        ListSubscriptionsResponse {
            subscriptions,
        }
    }
}

