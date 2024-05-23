pub mod protocol;

#[derive(Clone, Debug)]
pub struct Message {
    pub message_type: u16,
    pub message: protocol::Message,
    pub completed: bool,
    pub requires_ack: Option<bool>,
}
