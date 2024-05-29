# DetailedDeploymentResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**services** | [**Vec<models::ServiceMetadata>**](ServiceMetadata.md) | List of services exposed by this deployment. | 
**uri** | **String** |  | 
**protocol_type** | [**models::ProtocolType**](ProtocolType.md) |  | 
**additional_headers** | Option<**std::collections::HashMap<String, String>**> |  | [optional]
**created_at** | **String** |  | 
**min_protocol_version** | **i32** |  | 
**max_protocol_version** | **i32** |  | 
**arn** | **String** |  | 
**assume_role_arn** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


