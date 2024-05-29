# CreateSubscriptionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**source** | **String** | Source uri. Accepted forms:  * `kafka://<cluster_name>/<topic_name>`, e.g. `kafka://my-cluster/my-topic` | 
**sink** | **String** | Sink uri. Accepted forms:  * `service://<service_name>/<service_name>`, e.g. `service://Counter/count` | 
**options** | Option<**std::collections::HashMap<String, String>**> | Additional options to apply to the subscription. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


