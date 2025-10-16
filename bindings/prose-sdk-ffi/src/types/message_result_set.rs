// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{Message, MessageId};
use prose_core_client::dtos::MessageResultSet as CoreMessageResultSet;

#[derive(uniffi::Record)]
pub struct MessageResultSet {
    /// The requested messages in the order from oldest to newest.
    pub messages: Vec<Message>,
    /// Can be used to load more messages. `last_message_id` might not be contained in `messages`.
    /// If not set there are no more messages to load.
    pub last_message_id: Option<MessageId>,
}

impl From<CoreMessageResultSet> for MessageResultSet {
    fn from(value: CoreMessageResultSet) -> Self {
        MessageResultSet {
            messages: value.messages.into_iter().map(Into::into).collect(),
            last_message_id: value.last_message_id.map(Into::into),
        }
    }
}
