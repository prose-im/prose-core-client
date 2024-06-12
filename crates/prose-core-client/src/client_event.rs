// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};

use prose_xmpp::ConnectionError;

use crate::app::dtos::RoomEnvelope;
use crate::domain::messaging::models::MessageId;
use crate::domain::shared::models::UserId;

#[derive(Clone, PartialEq)]
pub enum ClientEvent {
    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// The contents of the sidebar have changed.
    SidebarChanged,

    /// Infos about a contact have changed.
    ContactChanged { ids: Vec<UserId> },

    /// Contacts were added, removed or their subscription status changed.
    ContactListChanged,

    /// A presence subscription request was either added or removed.
    PresenceSubRequestsChanged,

    /// A user was blocked or unblocked.
    BlockListChanged,

    /// The avatar of a user changed.
    AvatarChanged { ids: Vec<UserId> },

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

    /// The room went offline, came back online and contains new messages.
    MessagesNeedReload,

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

impl Debug for ClientEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientEvent::ConnectionStatusChanged { event } => f
                .debug_struct("ConnectionStatusChanged")
                .field("event", &event)
                .finish(),
            ClientEvent::SidebarChanged => f.debug_struct("SidebarChanged").finish(),
            ClientEvent::ContactChanged { ids } => {
                f.debug_struct("ContactChanged").field("ids", &ids).finish()
            }
            ClientEvent::ContactListChanged => f.debug_struct("ContactListChanged").finish(),
            ClientEvent::PresenceSubRequestsChanged => {
                f.debug_struct("PresenceSubRequestsChanged").finish()
            }
            ClientEvent::BlockListChanged => f.debug_struct("BlockListChanged").finish(),
            ClientEvent::AvatarChanged { ids } => {
                f.debug_struct("AvatarChanged").field("ids", &ids).finish()
            }
            ClientEvent::AccountInfoChanged => f.debug_struct("AccountInfoChanged").finish(),
            ClientEvent::RoomChanged { room, r#type } => f
                .debug_struct("RoomChanged")
                .field("room", &room.to_generic_room().jid())
                .field("type", &r#type)
                .finish(),
        }
    }
}
