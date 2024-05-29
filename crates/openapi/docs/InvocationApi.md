# \InvocationApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**terminate_invocation**](InvocationApi.md#terminate_invocation) | **DELETE** /invocations/{invocation_id} | Terminate an invocation



## terminate_invocation

> terminate_invocation(invocation_id, mode)
Terminate an invocation

Terminate the given invocation. By default, an invocation is terminated by gracefully cancelling it. This ensures virtual object state consistency. Alternatively, an invocation can be killed which does not guarantee consistency for virtual object instance state, in-flight invocations to other services, etc.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invocation_id** | **String** | Invocation identifier. | [required] |
**mode** | Option<[**TerminationMode**](.md)> | If cancel, it will gracefully terminate the invocation. If kill, it will terminate the invocation with a hard stop. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

