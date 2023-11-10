// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use prose_xmpp::ConnectionError;

use crate::app::services::RoomEnvelope;
use crate::domain::messaging::models::MessageId;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// The contents of the sidebar have changed.
    SidebarChanged,

    /// Infos about a contact have changed.
    ContactChanged { jid: BareJid },

    /// The avatar of a user changed.
    AvatarChanged { jid: BareJid },

    RoomChanged {
        room: RoomEnvelope,
        r#type: RoomEventType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomEventType {
    /// One or many messages were either received or sent.
    MessagesAppended { message_ids: Vec<MessageId> },

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    MessagesUpdated { message_ids: Vec<MessageId> },

    /// A message was deleted.
    MessagesDeleted { message_ids: Vec<MessageId> },

    /// A user in `conversation` started or stopped typing.
    ComposingUsersChanged,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}
