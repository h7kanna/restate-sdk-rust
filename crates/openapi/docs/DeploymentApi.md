# \DeploymentApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_deployment**](DeploymentApi.md#create_deployment) | **POST** /deployments | Create deployment
[**delete_deployment**](DeploymentApi.md#delete_deployment) | **DELETE** /deployments/{deployment} | Delete deployment
[**get_deployment**](DeploymentApi.md#get_deployment) | **GET** /deployments/{deployment} | Get deployment
[**list_deployments**](DeploymentApi.md#list_deployments) | **GET** /deployments | List deployments



## create_deployment

> models::RegisterDeploymentResponse create_deployment(register_deployment_request)
Create deployment

Create deployment. Restate will invoke the endpoint to gather additional information required for registration, such as the services exposed by the deployment. If the deployment is already registered, this method will fail unless `force` is set to `true`.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**register_deployment_request** | [**RegisterDeploymentRequest**](RegisterDeploymentRequest.md) |  | [required] |

### Return type

[**models::RegisterDeploymentResponse**](RegisterDeploymentResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_deployment

> delete_deployment(deployment, force)
Delete deployment

Delete deployment. Currently it's supported to remove a deployment only using the force flag

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment** | **String** | Deployment identifier | [required] |
**force** | Option<**bool**> | If true, the deployment will be forcefully deleted. This might break in-flight invocations, use with caution. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deployment

> models::DetailedDeploymentResponse get_deployment(deployment)
Get deployment

Get deployment metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**deployment** | **String** | Deployment identifier | [required] |

### Return type

[**models::DetailedDeploymentResponse**](DetailedDeploymentResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_deployments

> models::ListDeploymentsResponse list_deployments()
List deployments

List all registered deployments.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListDeploymentsResponse**](ListDeploymentsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

