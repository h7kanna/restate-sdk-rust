/*
 * Admin API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.2
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegisterDeploymentRequestAnyOf {
    /// Uri to use to discover/invoke the http deployment.
    #[serde(rename = "uri")]
    pub uri: String,
    /// Additional headers added to the discover/invoke requests to the deployment.
    #[serde(rename = "additional_headers", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub additional_headers: Option<Option<std::collections::HashMap<String, String>>>,
    /// If `true`, discovery will be attempted using a client that defaults to HTTP1.1 instead of a prior-knowledge HTTP2 client. HTTP2 may still be used for TLS servers that advertise HTTP2 support via ALPN. HTTP1.1 deployments will only work in request-response mode.
    #[serde(rename = "use_http_11", skip_serializing_if = "Option::is_none")]
    pub use_http_11: Option<bool>,
    /// If `true`, it will override, if existing, any deployment using the same `uri`. Beware that this can lead in-flight invocations to an unrecoverable error state.  By default, this is `true` but it might change in future to `false`.  See the [versioning documentation](https://docs.restate.dev/operate/versioning) for more information.
    #[serde(rename = "force", skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// If `true`, discovery will run but the deployment will not be registered. This is useful to see the impact of a new deployment before registering it.
    #[serde(rename = "dry_run", skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

impl RegisterDeploymentRequestAnyOf {
    pub fn new(uri: String) -> RegisterDeploymentRequestAnyOf {
        RegisterDeploymentRequestAnyOf {
            uri,
            additional_headers: None,
            use_http_11: None,
            force: None,
            dry_run: None,
        }
    }
}

