# RegisterDeploymentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uri** | **String** | Uri to use to discover/invoke the http deployment. | 
**additional_headers** | Option<**std::collections::HashMap<String, String>**> | Additional headers added to the discover/invoke requests to the deployment. | [optional]
**force** | Option<**bool**> | If `true`, it will override, if existing, any deployment using the same `uri`. Beware that this can lead in-flight invocations to an unrecoverable error state.  By default, this is `true` but it might change in future to `false`.  See the [versioning documentation](https://docs.restate.dev/operate/versioning) for more information. | [optional][default to true]
**dry_run** | Option<**bool**> | If `true`, discovery will run but the deployment will not be registered. This is useful to see the impact of a new deployment before registering it. | [optional][default to false]
**arn** | **String** | ARN to use to discover/invoke the lambda deployment. | 
**assume_role_arn** | Option<**String**> | Optional ARN of a role to assume when invoking the addressed Lambda, to support role chaining | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


