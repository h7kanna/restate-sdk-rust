/*
 * Admin API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.2
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize};
use crate::{apis::ResponseContent, models};
use super::{Error, configuration};


/// struct for typed errors of method [`get_service`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetServiceError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_services`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListServicesError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`modify_service`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModifyServiceError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`modify_service_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModifyServiceStateError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}


/// Get a registered service.
pub async fn get_service(configuration: &configuration::Configuration, service: &str) -> Result<models::ServiceMetadata, Error<GetServiceError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/services/{service}", local_var_configuration.base_path, service=crate::apis::urlencode(service));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetServiceError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// List all registered services.
pub async fn list_services(configuration: &configuration::Configuration, ) -> Result<models::ListServicesResponse, Error<ListServicesError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/services", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListServicesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Modify a registered service.
pub async fn modify_service(configuration: &configuration::Configuration, service: &str, modify_service_request: models::ModifyServiceRequest) -> Result<models::ServiceMetadata, Error<ModifyServiceError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/services/{service}", local_var_configuration.base_path, service=crate::apis::urlencode(service));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::PATCH, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&modify_service_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ModifyServiceError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Modify service state
pub async fn modify_service_state(configuration: &configuration::Configuration, service: &str, modify_service_state_request: models::ModifyServiceStateRequest) -> Result<(), Error<ModifyServiceStateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/services/{service}/state", local_var_configuration.base_path, service=crate::apis::urlencode(service));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&modify_service_state_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<ModifyServiceStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

