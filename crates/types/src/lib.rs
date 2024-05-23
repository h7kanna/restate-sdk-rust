pub mod protocol;
pub mod service_protocol;
pub mod endpoint_manifest;
pub mod invocation;
pub mod identifiers;
pub mod errors;
pub mod journal;
pub mod time;

#[derive(Clone, Debug)]
pub struct Message {
    pub message_type: u16,
    pub message: protocol::Message,
    pub completed: bool,
    pub requires_ack: Option<bool>,
}
