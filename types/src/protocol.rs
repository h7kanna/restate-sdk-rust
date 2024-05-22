use restate_sdk_protos::*;

// pub the protocol message types as defined by the restate protocol.
pub const START_MESSAGE_TYPE: u16 = 0x0000;
pub const COMPLETION_MESSAGE_TYPE: u16 = 0x0001;
pub const SUSPENSION_MESSAGE_TYPE: u16 = 0x0002;
pub const ERROR_MESSAGE_TYPE: u16 = 0x0003;
pub const ENTRY_ACK_MESSAGE_TYPE: u16 = 0x0004;
pub const END_MESSAGE_TYPE: u16 = 0x0005;
pub const INPUT_ENTRY_MESSAGE_TYPE: u16 = 0x0400;
pub const OUTPUT_ENTRY_MESSAGE_TYPE: u16 = 0x0401;
pub const GET_STATE_ENTRY_MESSAGE_TYPE: u16 = 0x0800;
pub const SET_STATE_ENTRY_MESSAGE_TYPE: u16 = 0x0801;
pub const CLEAR_STATE_ENTRY_MESSAGE_TYPE: u16 = 0x0802;
pub const CLEAR_ALL_STATE_ENTRY_MESSAGE_TYPE: u16 = 0x0803;
pub const GET_STATE_KEYS_ENTRY_MESSAGE_TYPE: u16 = 0x0804;
pub const SLEEP_ENTRY_MESSAGE_TYPE: u16 = 0x0c00;
pub const INVOKE_ENTRY_MESSAGE_TYPE: u16 = 0x0c01;
pub const BACKGROUND_INVOKE_ENTRY_MESSAGE_TYPE: u16 = 0x0c02;
pub const AWAKEABLE_ENTRY_MESSAGE_TYPE: u16 = 0x0c03;
pub const COMPLETE_AWAKEABLE_ENTRY_MESSAGE_TYPE: u16 = 0x0c04;

pub const AWAKEABLE_IDENTIFIER_PREFIX: &str = "prom_1";

pub const SIDE_EFFECT_ENTRY_MESSAGE_TYPE: u16 = 0x0c00 + 5;

#[derive(Debug, Clone)]
pub enum Message {
    AwakeableEntryMessage(u16, AwakeableEntryMessage),
    OneWayCallEntryMessage(u16, OneWayCallEntryMessage),
    ClearStateEntryMessage(u16, ClearStateEntryMessage),
    ClearAllStateEntryMessage(u16, ClearStateEntryMessage),
    CompleteAwakeableEntryMessage(u16, CompleteAwakeableEntryMessage),
    CompletionMessage(u16, CompletionMessage),
    EntryAckMessage(u16, EntryAckMessage),
    ErrorMessage(u16, ErrorMessage),
    EndMessage(u16, EndMessage),
    GetStateEntryMessage(u16, GetStateEntryMessage),
    GetStateKeysEntryMessage(u16, GetStateKeysEntryMessage),
    CallEntryMessage(u16, CallEntryMessage),
    OutputEntryMessage(u16, OutputEntryMessage),
    InputEntryMessage(u16, InputEntryMessage),
    SetStateEntryMessage(u16, SetStateEntryMessage),
    SleepEntryMessage(u16, SleepEntryMessage),
    StartMessage(u16, StartMessage),
    SuspensionMessage(u16, SuspensionMessage),
    RunEntryMessage(u16, RunEntryMessage),
}


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
