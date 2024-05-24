use restate_sdk_types::service_protocol::*;

pub const AWAKEABLE_IDENTIFIER_PREFIX: &str = "prom_1";

// These message types will trigger sending a suspension message from the runtime
// for each of the protocol modes
pub enum SuspensionTriggers {
    InvokeEntryMessageType,
    GetStateEntryMessageType,
    GetStateKeysEntryMessageType,
    AwakeableEntryMessageType,
    SleepEntryMessageType,
    CombinatorEntryMessage,
    SideEffectEntryMessageType,
}
