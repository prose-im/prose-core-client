// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Message;
use crate::dtos::MessageServerId;

#[derive(Debug, PartialEq)]
pub struct MessageResultSet {
    /// The requested messages in the order from oldest to newest.
    pub messages: Vec<Message>,
    /// Can be used to load more messages. `last_message_id` might not be contained in `messages`.
    /// If not set there are no more messages to load.
    pub last_message_id: Option<MessageServerId>,
}

impl IntoIterator for MessageResultSet {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}
