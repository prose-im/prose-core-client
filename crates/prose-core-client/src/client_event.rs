// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::ConnectionError;

use crate::app::services::RoomEnvelope;
use crate::domain::messaging::models::MessageId;
use crate::domain::shared::models::UserId;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// The contents of the sidebar have changed.
    SidebarChanged,

    /// Infos about a contact have changed.
    ContactChanged { id: UserId },

    /// The avatar of a user changed.
    AvatarChanged { id: UserId },

    /// Infos related to the logged-in user have changed.
    AccountInfoChanged,

    RoomChanged {
        room: RoomEnvelope,
        r#type: ClientRoomEventType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClientRoomEventType {
    /// One or many messages were either received or sent.
    MessagesAppended { message_ids: Vec<MessageId> },

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    MessagesUpdated { message_ids: Vec<MessageId> },

    /// A message was deleted.
    MessagesDeleted { message_ids: Vec<MessageId> },

    /// Attributes changed like name or topic.
    AttributesChanged,

    /// The list of participants has changed.
    ParticipantsChanged,

    /// A user in `conversation` started or stopped typing.
    ComposingUsersChanged,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}
