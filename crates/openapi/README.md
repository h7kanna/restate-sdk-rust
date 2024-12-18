# Rust API client for openapi

No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)


## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: 1.1.2
- Package version: 1.1.2
- Generator version: 7.9.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `openapi` and add the following to `Cargo.toml` under `[dependencies]`:

```
openapi = { path = "./openapi" }
```

## Documentation for API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*DeploymentApi* | [**create_deployment**](docs/DeploymentApi.md#create_deployment) | **POST** /deployments | Create deployment
*DeploymentApi* | [**delete_deployment**](docs/DeploymentApi.md#delete_deployment) | **DELETE** /deployments/{deployment} | Delete deployment
*DeploymentApi* | [**get_deployment**](docs/DeploymentApi.md#get_deployment) | **GET** /deployments/{deployment} | Get deployment
*DeploymentApi* | [**list_deployments**](docs/DeploymentApi.md#list_deployments) | **GET** /deployments | List deployments
*HealthApi* | [**health**](docs/HealthApi.md#health) | **GET** /health | Health check
*InvocationApi* | [**delete_invocation**](docs/InvocationApi.md#delete_invocation) | **DELETE** /invocations/{invocation_id} | Delete an invocation
*OpenapiApi* | [**openapi_spec**](docs/OpenapiApi.md#openapi_spec) | **GET** /openapi | OpenAPI specification
*ServiceApi* | [**get_service**](docs/ServiceApi.md#get_service) | **GET** /services/{service} | Get service
*ServiceApi* | [**list_services**](docs/ServiceApi.md#list_services) | **GET** /services | List services
*ServiceApi* | [**modify_service**](docs/ServiceApi.md#modify_service) | **PATCH** /services/{service} | Modify a service
*ServiceApi* | [**modify_service_state**](docs/ServiceApi.md#modify_service_state) | **POST** /services/{service}/state | Modify a service state
*ServiceHandlerApi* | [**get_service_handler**](docs/ServiceHandlerApi.md#get_service_handler) | **GET** /services/{service}/handlers/{handler} | Get service handler
*ServiceHandlerApi* | [**list_service_handlers**](docs/ServiceHandlerApi.md#list_service_handlers) | **GET** /services/{service}/handlers | List service handlers
*SubscriptionApi* | [**create_subscription**](docs/SubscriptionApi.md#create_subscription) | **POST** /subscriptions | Create subscription
*SubscriptionApi* | [**delete_subscription**](docs/SubscriptionApi.md#delete_subscription) | **DELETE** /subscriptions/{subscription} | Delete subscription
*SubscriptionApi* | [**get_subscription**](docs/SubscriptionApi.md#get_subscription) | **GET** /subscriptions/{subscription} | Get subscription
*SubscriptionApi* | [**list_subscriptions**](docs/SubscriptionApi.md#list_subscriptions) | **GET** /subscriptions | List subscriptions
*VersionApi* | [**version**](docs/VersionApi.md#version) | **GET** /version | Admin version information


## Documentation For Models

 - [CreateSubscriptionRequest](docs/CreateSubscriptionRequest.md)
 - [DeletionMode](docs/DeletionMode.md)
 - [DeploymentResponse](docs/DeploymentResponse.md)
 - [DeploymentResponseAnyOf](docs/DeploymentResponseAnyOf.md)
 - [DeploymentResponseAnyOf1](docs/DeploymentResponseAnyOf1.md)
 - [DetailedDeploymentResponse](docs/DetailedDeploymentResponse.md)
 - [ErrorDescriptionResponse](docs/ErrorDescriptionResponse.md)
 - [HandlerMetadata](docs/HandlerMetadata.md)
 - [HandlerMetadataType](docs/HandlerMetadataType.md)
 - [ListDeploymentsResponse](docs/ListDeploymentsResponse.md)
 - [ListServiceHandlersResponse](docs/ListServiceHandlersResponse.md)
 - [ListServicesResponse](docs/ListServicesResponse.md)
 - [ListSubscriptionsResponse](docs/ListSubscriptionsResponse.md)
 - [ModifyServiceRequest](docs/ModifyServiceRequest.md)
 - [ModifyServiceStateRequest](docs/ModifyServiceStateRequest.md)
 - [ProtocolType](docs/ProtocolType.md)
 - [RegisterDeploymentRequest](docs/RegisterDeploymentRequest.md)
 - [RegisterDeploymentRequestAnyOf](docs/RegisterDeploymentRequestAnyOf.md)
 - [RegisterDeploymentRequestAnyOf1](docs/RegisterDeploymentRequestAnyOf1.md)
 - [RegisterDeploymentResponse](docs/RegisterDeploymentResponse.md)
 - [ServiceMetadata](docs/ServiceMetadata.md)
 - [ServiceNameRevPair](docs/ServiceNameRevPair.md)
 - [ServiceType](docs/ServiceType.md)
 - [SubscriptionResponse](docs/SubscriptionResponse.md)
 - [VersionInformation](docs/VersionInformation.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author



