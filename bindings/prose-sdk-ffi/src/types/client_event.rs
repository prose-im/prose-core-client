// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{ConnectionError, RoomEnvelope};
use crate::{MessageId, UserId};
use prose_core_client::{
    ClientEvent as CoreClientEvent, ClientRoomEventType as CoreClientRoomEventType,
    ConnectionEvent as CoreConnectionEvent,
};

#[derive(uniffi::Enum)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}

#[derive(uniffi::Enum)]
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

    /// Infos related to the server/workspace have changed.
    WorkspaceInfoChanged,

    /// The avatar of the server/workspace has changed.
    WorkspaceIconChanged,

    RoomChanged {
        room: RoomEnvelope,
        r#type: ClientRoomEventType,
    },
}

#[derive(uniffi::Enum)]
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

impl From<CoreConnectionEvent> for ConnectionEvent {
    fn from(value: CoreConnectionEvent) -> Self {
        match value {
            CoreConnectionEvent::Connect => ConnectionEvent::Connect,
            CoreConnectionEvent::Disconnect { error } => ConnectionEvent::Disconnect {
                error: error.map(Into::into),
            },
        }
    }
}

impl From<CoreClientRoomEventType> for ClientRoomEventType {
    fn from(value: CoreClientRoomEventType) -> Self {
        match value {
            CoreClientRoomEventType::MessagesAppended { message_ids } => {
                ClientRoomEventType::MessagesAppended {
                    message_ids: message_ids.into_iter().map(Into::into).collect(),
                }
            }
            CoreClientRoomEventType::MessagesUpdated { message_ids } => {
                ClientRoomEventType::MessagesUpdated {
                    message_ids: message_ids.into_iter().map(Into::into).collect(),
                }
            }
            CoreClientRoomEventType::MessagesDeleted { message_ids } => {
                ClientRoomEventType::MessagesDeleted {
                    message_ids: message_ids.into_iter().map(Into::into).collect(),
                }
            }
            CoreClientRoomEventType::MessagesNeedReload => ClientRoomEventType::MessagesNeedReload,
            CoreClientRoomEventType::AttributesChanged => ClientRoomEventType::AttributesChanged,
            CoreClientRoomEventType::ParticipantsChanged => {
                ClientRoomEventType::ParticipantsChanged
            }
            CoreClientRoomEventType::ComposingUsersChanged => {
                ClientRoomEventType::ComposingUsersChanged
            }
        }
    }
}

impl From<CoreClientEvent> for ClientEvent {
    fn from(value: CoreClientEvent) -> Self {
        match value {
            CoreClientEvent::ConnectionStatusChanged { event } => {
                ClientEvent::ConnectionStatusChanged {
                    event: event.into(),
                }
            }
            CoreClientEvent::SidebarChanged => ClientEvent::SidebarChanged,
            CoreClientEvent::ContactChanged { ids } => ClientEvent::ContactChanged {
                ids: ids.into_iter().map(Into::into).collect(),
            },
            CoreClientEvent::ContactListChanged => ClientEvent::ContactListChanged,
            CoreClientEvent::PresenceSubRequestsChanged => ClientEvent::PresenceSubRequestsChanged,
            CoreClientEvent::BlockListChanged => ClientEvent::BlockListChanged,
            CoreClientEvent::AvatarChanged { ids } => ClientEvent::AvatarChanged {
                ids: ids.into_iter().map(Into::into).collect(),
            },
            CoreClientEvent::AccountInfoChanged => ClientEvent::AccountInfoChanged,
            CoreClientEvent::WorkspaceInfoChanged => ClientEvent::WorkspaceInfoChanged,
            CoreClientEvent::WorkspaceIconChanged => ClientEvent::WorkspaceIconChanged,
            CoreClientEvent::RoomChanged { room, r#type } => ClientEvent::RoomChanged {
                room: room.into(),
                r#type: r#type.into(),
            },
        }
    }
}
