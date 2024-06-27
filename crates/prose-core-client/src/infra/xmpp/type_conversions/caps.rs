// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sha1::{Digest, Sha1};
use xmpp_parsers::hashes::{Algo, Hash};

use crate::domain::general::models::Capabilities;

impl From<&Capabilities> for xmpp_parsers::caps::Caps {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::caps::Caps::new(
            value.node.clone(),
            Hash {
                algo: Algo::Sha_1,
                hash: Sha1::digest(value.ver_string.as_bytes()).to_vec(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::ns;

    use crate::domain::general::models::Feature;
    use crate::domain::user_info::models::PROSE_IM_NODE;

    use super::*;

    #[test]
    fn test_ver_string_exodus() {
        let caps = Capabilities::new(
            "Exodus 0.9.1",
            "http://code.google.com/p/exodus",
            vec![
                Feature::Name(ns::MUC),
                Feature::Name(ns::CAPS),
                Feature::Name(ns::DISCO_ITEMS),
                Feature::Name(ns::DISCO_INFO),
            ],
        );

        assert_eq!(caps.ver_string, "client/pc/en/Exodus 0.9.1<http://jabber.org/protocol/caps<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/disco#items<http://jabber.org/protocol/muc<");
    }

    #[test]
    fn test_ver_string_prose() {
        let caps = Capabilities::new(
            "Prose",
            PROSE_IM_NODE,
            vec![
                Feature::Name(ns::JABBER_CLIENT),
                Feature::Name(ns::AVATAR_DATA),
                Feature::Name(ns::AVATAR_METADATA),
                Feature::Name(ns::CHATSTATES),
                Feature::Name(ns::DISCO_INFO),
                Feature::Name(ns::RSM),
                Feature::Name(ns::CAPS),
                Feature::Name(ns::PING),
                Feature::Name(ns::PUBSUB),
                Feature::Name(ns::PUBSUB_EVENT),
                Feature::Name(ns::ROSTER),
                Feature::Name(ns::REACTIONS),
                Feature::Name(ns::RECEIPTS),
                Feature::Name(ns::CHAT_MARKERS),
                Feature::Name(ns::MESSAGE_CORRECT),
                Feature::Name(ns::RETRACT),
                Feature::Name(ns::FASTEN),
                Feature::Name(ns::DELAY),
                Feature::Name(ns::FALLBACK),
                Feature::Name(ns::HINTS),
                Feature::Name(ns::MAM),
                Feature::Name(ns::TIME),
                Feature::Name(ns::VERSION),
                Feature::Name(ns::LAST_ACTIVITY),
                Feature::Name(ns::USER_ACTIVITY),
                Feature::Name(ns::VCARD4),
                Feature::Notify(ns::PUBSUB),
                Feature::Notify(ns::USER_ACTIVITY),
                Feature::Notify(ns::AVATAR_METADATA),
                Feature::Notify(ns::VCARD4),
            ],
        );

        assert_eq!(caps.ver_string, "client/pc/en/Prose<http://jabber.org/protocol/activity<http://jabber.org/protocol/activity+notify<http://jabber.org/protocol/caps<http://jabber.org/protocol/chatstates<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/pubsub<http://jabber.org/protocol/pubsub#event<http://jabber.org/protocol/pubsub+notify<http://jabber.org/protocol/rsm<jabber:client<jabber:iq:last<jabber:iq:roster<jabber:iq:version<urn:ietf:params:xml:ns:vcard-4.0<urn:ietf:params:xml:ns:vcard-4.0+notify<urn:xmpp:avatar:data<urn:xmpp:avatar:metadata<urn:xmpp:avatar:metadata+notify<urn:xmpp:chat-markers:0<urn:xmpp:delay<urn:xmpp:fallback:0<urn:xmpp:fasten:0<urn:xmpp:hints<urn:xmpp:mam:2<urn:xmpp:message-correct:0<urn:xmpp:message-retract:0<urn:xmpp:ping<urn:xmpp:reactions:0<urn:xmpp:receipts<urn:xmpp:time<");
    }
}
