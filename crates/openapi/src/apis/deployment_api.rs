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


/// struct for typed errors of method [`create_deployment`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateDeploymentError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_deployment`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteDeploymentError {
    Status501(),
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_deployment`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetDeploymentError {
    Status400(models::ErrorDescriptionResponse),
    Status403(models::ErrorDescriptionResponse),
    Status404(models::ErrorDescriptionResponse),
    Status409(models::ErrorDescriptionResponse),
    Status500(models::ErrorDescriptionResponse),
    Status503(models::ErrorDescriptionResponse),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_deployments`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListDeploymentsError {
    UnknownValue(serde_json::Value),
}


/// Create deployment. Restate will invoke the endpoint to gather additional information required for registration, such as the services exposed by the deployment. If the deployment is already registered, this method will fail unless `force` is set to `true`.
pub async fn create_deployment(configuration: &configuration::Configuration, register_deployment_request: models::RegisterDeploymentRequest) -> Result<models::RegisterDeploymentResponse, Error<CreateDeploymentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/deployments", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&register_deployment_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateDeploymentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Delete deployment. Currently it's supported to remove a deployment only using the force flag
pub async fn delete_deployment(configuration: &configuration::Configuration, deployment: &str, force: Option<bool>) -> Result<(), Error<DeleteDeploymentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/deployments/{deployment}", local_var_configuration.base_path, deployment=crate::apis::urlencode(deployment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = force {
        local_var_req_builder = local_var_req_builder.query(&[("force", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<DeleteDeploymentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get deployment metadata
pub async fn get_deployment(configuration: &configuration::Configuration, deployment: &str) -> Result<models::DetailedDeploymentResponse, Error<GetDeploymentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/deployments/{deployment}", local_var_configuration.base_path, deployment=crate::apis::urlencode(deployment));
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
        let local_var_entity: Option<GetDeploymentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// List all registered deployments.
pub async fn list_deployments(configuration: &configuration::Configuration, ) -> Result<models::ListDeploymentsResponse, Error<ListDeploymentsError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/deployments", local_var_configuration.base_path);
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
        let local_var_entity: Option<ListDeploymentsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

