pub mod protocol;
pub mod service_protocol;
pub mod endpoint_manifest;

#[derive(Clone, Debug)]
pub struct Message {
    pub message_type: u16,
    pub message: protocol::Message,
    pub completed: bool,
    pub requires_ack: Option<bool>,
}
