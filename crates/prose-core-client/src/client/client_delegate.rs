// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use prose_xmpp::ConnectionError;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{ConnectedRoom, MessageId};
use crate::Client;

#[derive(Debug, PartialEq)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}

pub enum ClientEvent<D: DataCache + 'static, A: AvatarCache + 'static> {
    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// The number of rooms changed.
    RoomsChanged,

    /// Infos about a contact have changed.
    ContactChanged { jid: BareJid },

    /// The avatar of a user changed.
    AvatarChanged { jid: BareJid },

    /// A user in `conversation` started or stopped typing.
    ComposingUsersChanged { room: ConnectedRoom<D, A> },

    /// One or many messages were either received or sent.
    MessagesAppended {
        room: ConnectedRoom<D, A>,
        message_ids: Vec<MessageId>,
    },

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    MessagesUpdated {
        room: ConnectedRoom<D, A>,
        message_ids: Vec<MessageId>,
    },

    /// A message was deleted.
    MessagesDeleted {
        room: ConnectedRoom<D, A>,
        message_ids: Vec<MessageId>,
    },
}

pub trait ClientDelegate<D: DataCache, A: AvatarCache>: SendUnlessWasm + SyncUnlessWasm {
    fn handle_event(&self, client: Client<D, A>, event: ClientEvent<D, A>);
}
