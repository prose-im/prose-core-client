// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
