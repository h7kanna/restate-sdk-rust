# \InvocationApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_invocation**](InvocationApi.md#delete_invocation) | **DELETE** /invocations/{invocation_id} | Delete an invocation



## delete_invocation

> delete_invocation(invocation_id, mode)
Delete an invocation

Delete the given invocation. By default, an invocation is terminated by gracefully cancelling it. This ensures virtual object state consistency. Alternatively, an invocation can be killed which does not guarantee consistency for virtual object instance state, in-flight invocations to other services, etc. A stored completed invocation can also be purged

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invocation_id** | **String** | Invocation identifier. | [required] |
**mode** | Option<[**DeletionMode**](.md)> | If cancel, it will gracefully terminate the invocation. If kill, it will terminate the invocation with a hard stop. If purge, it will only cleanup the response for completed invocations, and leave unaffected an in-flight invocation. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

