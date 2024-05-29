# \ServiceHandlerApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_service_handler**](ServiceHandlerApi.md#get_service_handler) | **GET** /services/{service}/handlers/{handler} | Get service handler
[**list_service_handlers**](ServiceHandlerApi.md#list_service_handlers) | **GET** /services/{service}/handlers | List service handlers



## get_service_handler

> models::HandlerMetadata get_service_handler(service, handler)
Get service handler

Get the handler of a service

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service** | **String** | Fully qualified service name. | [required] |
**handler** | **String** | Handler name. | [required] |

### Return type

[**models::HandlerMetadata**](HandlerMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_service_handlers

> models::ListServiceHandlersResponse list_service_handlers(service)
List service handlers

List all the handlers of the given service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service** | **String** | Fully qualified service name. | [required] |

### Return type

[**models::ListServiceHandlersResponse**](ListServiceHandlersResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

