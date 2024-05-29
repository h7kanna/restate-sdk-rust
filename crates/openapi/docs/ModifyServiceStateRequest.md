# ModifyServiceStateRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version** | Option<**String**> | If set, the latest version of the state is compared with this value and the operation will fail when the versions differ. | [optional]
**object_key** | **String** | To what virtual object key to apply this change | 
**new_state** | [**std::collections::HashMap<String, Vec<i32>>**](Vec.md) | The new state to replace the previous state with | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


