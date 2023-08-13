// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use prose_xmpp::{ConnectionError, SendUnlessWasm, SyncUnlessWasm};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::MessageId;
use crate::Client;

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: Option<ConnectionError> },
}

#[derive(Debug)]
pub enum ClientEvent {
    /// A user in `conversation` started or stopped typing.
    ComposingUsersChanged { conversation: BareJid },

    /// The status of the connection has changed.
    ConnectionStatusChanged { event: ConnectionEvent },

    /// Infos about a contact have changed.
    ContactChanged { jid: BareJid },

    /// The avatar of a user changed.
    AvatarChanged { jid: BareJid },

    /// One or many messages were either received or sent.
    MessagesAppended {
        conversation: BareJid,
        message_ids: Vec<MessageId>,
    },

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    MessagesUpdated {
        conversation: BareJid,
        message_ids: Vec<MessageId>,
    },

    /// A message was deleted.
    MessagesDeleted {
        conversation: BareJid,
        message_ids: Vec<MessageId>,
    },
}

pub trait ClientDelegate<D: DataCache, A: AvatarCache>: SendUnlessWasm + SyncUnlessWasm {
    fn handle_event(&self, client: Client<D, A>, event: ClientEvent);
}
