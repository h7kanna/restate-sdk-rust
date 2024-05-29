# \SubscriptionApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_subscription**](SubscriptionApi.md#create_subscription) | **POST** /subscriptions | Create subscription
[**delete_subscription**](SubscriptionApi.md#delete_subscription) | **DELETE** /subscriptions/{subscription} | Delete subscription
[**get_subscription**](SubscriptionApi.md#get_subscription) | **GET** /subscriptions/{subscription} | Get subscription
[**list_subscriptions**](SubscriptionApi.md#list_subscriptions) | **GET** /subscriptions | List subscriptions



## create_subscription

> models::SubscriptionResponse create_subscription(create_subscription_request)
Create subscription

Create subscription.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_subscription_request** | [**CreateSubscriptionRequest**](CreateSubscriptionRequest.md) |  | [required] |

### Return type

[**models::SubscriptionResponse**](SubscriptionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_subscription

> delete_subscription(subscription)
Delete subscription

Delete subscription.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**subscription** | **String** | Subscription identifier | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_subscription

> models::SubscriptionResponse get_subscription(subscription)
Get subscription

Get subscription

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**subscription** | **String** | Subscription identifier | [required] |

### Return type

[**models::SubscriptionResponse**](SubscriptionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_subscriptions

> models::ListSubscriptionsResponse list_subscriptions(sink, source)
List subscriptions

List all subscriptions.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sink** | Option<**String**> | Filter by the exact specified sink. |  |
**source** | Option<**String**> | Filter by the exact specified source. |  |

### Return type

[**models::ListSubscriptionsResponse**](ListSubscriptionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

