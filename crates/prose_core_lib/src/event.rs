use jid::Jid;
use xmpp_parsers::caps::Caps;
use xmpp_parsers::presence::Presence;

use crate::connector::ConnectionError;
use crate::mods::chat::Carbon;
use crate::stanza::{avatar, Message, VCard4};

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Disconnected {
        error: ConnectionError,
    },

    DiscoInfoQuery {
        from: Jid,
        node: String,
    },
    CapsPresence {
        from: Jid,
        caps: Caps,
    },

    Message(Message),
    Carbon(Carbon),
    Sent(Message),

    Vcard {
        from: Jid,
        vcard: VCard4,
    },
    AvatarMetadata {
        from: Jid,
        metadata: avatar::Metadata,
    },
    Presence(Presence),
}
