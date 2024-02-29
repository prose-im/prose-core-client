// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use prose_utils::id_string;

id_string!(MessageId);
id_string!(StanzaId);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type", content = "id")]
pub enum MessageTargetId {
    MessageId(MessageId),
    StanzaId(StanzaId),
}

impl From<MessageId> for MessageTargetId {
    fn from(value: MessageId) -> Self {
        Self::MessageId(value)
    }
}

impl From<StanzaId> for MessageTargetId {
    fn from(value: StanzaId) -> Self {
        Self::StanzaId(value)
    }
}
