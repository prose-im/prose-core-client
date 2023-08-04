use crate::types::Message;
use prose_core_client::types::{Message as ProseMessage, Page};

#[derive(uniffi::Record)]
pub struct MessagesPage {
    pub messages: Vec<Message>,
    pub is_complete: bool,
}

impl From<Page<ProseMessage>> for MessagesPage {
    fn from(value: Page<ProseMessage>) -> Self {
        MessagesPage {
            messages: value.items.into_iter().map(Into::into).collect(),
            is_complete: value.is_complete,
        }
    }
}
