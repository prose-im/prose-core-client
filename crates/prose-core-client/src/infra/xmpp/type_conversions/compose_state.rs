// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::ComposeState;
use xmpp_parsers::chatstates::ChatState;

impl From<ChatState> for ComposeState {
    fn from(value: ChatState) -> Self {
        match value {
            ChatState::Composing => ComposeState::Composing,
            ChatState::Active | ChatState::Gone | ChatState::Inactive | ChatState::Paused => {
                ComposeState::Idle
            }
        }
    }
}
