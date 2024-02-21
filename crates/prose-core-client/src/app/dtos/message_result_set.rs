// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Message;

#[derive(Debug)]
pub struct MessageResultSet {
    pub messages: Vec<Message>,
    /// Are there more messages or is this the last page?
    pub is_last: bool,
}

impl IntoIterator for MessageResultSet {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}
