// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::ConnectionError;
use crate::uniffi_types::{MessageId, JID};

#[derive(uniffi::Enum)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}

#[derive(uniffi::Enum)]
pub enum ClientEvent {
    /// A user in `conversation` started or stopped typing.
    ComposingUsersChanged { conversation: JID },

    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// Infos about a contact have changed.
    ContactChanged { jid: JID },

    /// The avatar of a user changed.
    AvatarChanged { jid: JID },

    /// One or many messages were either received or sent.
    MessagesAppended {
        conversation: JID,
        message_ids: Vec<MessageId>,
    },

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    MessagesUpdated {
        conversation: JID,
        message_ids: Vec<MessageId>,
    },

    /// A message was deleted.
    MessagesDeleted {
        conversation: JID,
        message_ids: Vec<MessageId>,
    },
}

impl From<prose_core_client::ClientEvent> for ClientEvent {
    fn from(_value: prose_core_client::ClientEvent) -> Self {
        todo!("FIXME")
        // match value {
        //     ProseClientEvent::ComposingUsersChanged { room } => {
        //         ClientEvent::ComposingUsersChanged {
        //             conversation: conversation.into(),
        //         }
        //     }
        //     ProseClientEvent::ConnectionStatusChanged { event } => {
        //         ClientEvent::ConnectionStatusChanged { event }
        //     }
        //     ProseClientEvent::ContactChanged { jid } => {
        //         ClientEvent::ContactChanged { jid: jid.into() }
        //     }
        //     ProseClientEvent::RoomsChanged => todo!("Handle RoomsChanged event"),
        //     ProseClientEvent::AvatarChanged { jid } => {
        //         ClientEvent::AvatarChanged { jid: jid.into() }
        //     }
        //     ProseClientEvent::MessagesAppended {
        //         conversation,
        //         message_ids,
        //     } => ClientEvent::MessagesAppended {
        //         conversation: conversation.into(),
        //         message_ids,
        //     },
        //     ProseClientEvent::MessagesUpdated {
        //         conversation,
        //         message_ids,
        //     } => ClientEvent::MessagesUpdated {
        //         conversation: conversation.into(),
        //         message_ids,
        //     },
        //     ProseClientEvent::MessagesDeleted {
        //         conversation,
        //         message_ids,
        //     } => ClientEvent::MessagesDeleted {
        //         conversation: conversation.into(),
        //         message_ids,
        //     },
        // }
    }
}
