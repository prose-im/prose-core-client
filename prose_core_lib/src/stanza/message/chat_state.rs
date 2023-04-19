use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString, Clone, serde::Serialize, serde::Deserialize)]
#[strum(serialize_all = "lowercase")]
pub enum ChatState {
    /// User is actively participating in the chat session.
    Active,
    /// User has not been actively participating in the chat session.
    Composing,
    /// User has effectively ended their participation in the chat session.
    Gone,
    /// User is composing a message.
    Inactive,
    /// User had been composing but now has stopped.
    Paused,
}
