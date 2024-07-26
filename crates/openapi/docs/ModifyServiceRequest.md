# ModifyServiceRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**public** | Option<**bool**> | If true, the service can be invoked through the ingress. If false, the service can be invoked only from another Restate service. | [optional]
**idempotency_retention** | Option<**String**> | Modify the retention of idempotent requests for this service.  Can be configured using the [`humantime`](https://docs.rs/humantime/latest/humantime/fn.parse_duration.html) format or the ISO8601. | [optional]
**workflow_completion_retention** | Option<**String**> | Modify the retention of the workflow completion. This can be modified only for workflow services!  Can be configured using the [`humantime`](https://docs.rs/humantime/latest/humantime/fn.parse_duration.html) format or the ISO8601. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


