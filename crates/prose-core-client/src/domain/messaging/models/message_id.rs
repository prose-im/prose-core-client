// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use prose_utils::id_string;

id_string!(
    /// The ID assigned by a client sending the message. It is not guaranteed to be unique.
    MessageRemoteId
);

id_string!(
    /// The ID assigned by the server to the message.
    MessageServerId
);

id_string!(
    /// The ID assigned to the message by us locally.
    MessageId
);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type", content = "id")]
pub enum MessageTargetId {
    RemoteId(MessageRemoteId),
    ServerId(MessageServerId),
}

impl From<MessageRemoteId> for MessageTargetId {
    fn from(value: MessageRemoteId) -> Self {
        Self::RemoteId(value)
    }
}

impl From<MessageServerId> for MessageTargetId {
    fn from(value: MessageServerId) -> Self {
        Self::ServerId(value)
    }
}
