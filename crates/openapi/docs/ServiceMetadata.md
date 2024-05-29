# ServiceMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | Fully qualified name of the service | 
**handlers** | [**Vec<models::HandlerMetadata>**](HandlerMetadata.md) |  | 
**ty** | [**models::ServiceType**](ServiceType.md) |  | 
**deployment_id** | **String** | Deployment exposing the latest revision of the service. | 
**revision** | **i32** | Latest revision of the service. | 
**public** | **bool** | If true, the service can be invoked through the ingress. If false, the service can be invoked only from another Restate service. | 
**idempotency_retention** | **String** | The retention duration of idempotent requests for this service. | 
**workflow_completion_retention** | Option<**String**> | The retention duration of workflows. Only available on workflow services. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


