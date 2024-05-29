# \ServiceApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_service**](ServiceApi.md#get_service) | **GET** /services/{service} | Get service
[**list_services**](ServiceApi.md#list_services) | **GET** /services | List services
[**modify_service**](ServiceApi.md#modify_service) | **PATCH** /services/{service} | Modify a service
[**modify_service_state**](ServiceApi.md#modify_service_state) | **POST** /services/{service}/state | Modify a service state



## get_service

> models::ServiceMetadata get_service(service)
Get service

Get a registered service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service** | **String** | Fully qualified service name. | [required] |

### Return type

[**models::ServiceMetadata**](ServiceMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_services

> models::ListServicesResponse list_services()
List services

List all registered services.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ListServicesResponse**](ListServicesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_service

> models::ServiceMetadata modify_service(service, modify_service_request)
Modify a service

Modify a registered service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service** | **String** | Fully qualified service name. | [required] |
**modify_service_request** | [**ModifyServiceRequest**](ModifyServiceRequest.md) |  | [required] |

### Return type

[**models::ServiceMetadata**](ServiceMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## modify_service_state

> modify_service_state(service, modify_service_state_request)
Modify a service state

Modify service state

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service** | **String** | Fully qualified service name. | [required] |
**modify_service_state_request** | [**ModifyServiceStateRequest**](ModifyServiceStateRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

