// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};
use xmpp_parsers::hashes::{Algo, Hash};

pub type Namespace = &'static str;

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub node: String,
    pub identity: Identity,
    pub features: Vec<Feature>,
    pub ver_string: String,
    pub ver_hash: String,
}

impl Capabilities {
    pub fn new(
        client_name: impl Into<String>,
        client_website: impl Into<String>,
        features: impl IntoIterator<Item = Feature>,
    ) -> Self {
        let identity = Identity {
            category: "client".to_string(),
            kind: "pc".to_string(),
            name: client_name.into(),
            lang: "".to_string(),
        };
        let features: Vec<Feature> = features.into_iter().collect();

        let ver_string = Capabilities::ver_string(&identity, features.iter());
        let ver_hash = Capabilities::ver_hash(&ver_string);

        Capabilities {
            node: client_website.into(),
            identity,
            features,
            ver_string,
            ver_hash,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub category: String,
    pub kind: String,
    pub name: String,
    pub lang: String,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Name(Namespace),
    Notify(Namespace),
}

impl Display for Feature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(namespace) => {
                write!(f, "{}", namespace)
            }
            Self::Notify(namespace) => {
                write!(f, "{}+notify", namespace)
            }
        }
    }
}

impl Capabilities {
    fn ver_string<'a>(
        identity: &Identity,
        features: impl Iterator<Item = &'a Feature>,
    ) -> String {
        let mut string = format!(
            "{}/{}/{}/{}<",
            identity.category, identity.kind, identity.lang, identity.name
        );

        let mut features: Vec<String> = features.into_iter().map(|f| f.to_string()).collect();
        features.sort();

        for feat in features {
            string.push_str(&feat);
            string.push_str("<");
        }

        string
    }

    fn ver_hash(ver_string: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(ver_string.as_bytes());
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

impl From<&Capabilities> for xmpp_parsers::disco::DiscoInfoResult {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::disco::DiscoInfoResult {
            node: Some(format!("{}#{}", value.node, value.ver_hash)),
            identities: vec![(&value.identity).into()],
            features: value.features.iter().map(Into::into).collect(),
            extensions: vec![],
        }
    }
}

impl From<&Capabilities> for xmpp_parsers::caps::Caps {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::caps::Caps::new(
            "sha-1",
            Hash {
                algo: Algo::Sha_1,
                hash: value.node.as_bytes().to_vec(),
            },
        )
    }
}

impl From<&Identity> for xmpp_parsers::disco::Identity {
    fn from(value: &Identity) -> Self {
        xmpp_parsers::disco::Identity {
            category: value.category.clone(),
            type_: value.kind.clone(),
            lang: None,
            name: Some(value.name.clone()),
        }
    }
}

impl From<&Feature> for xmpp_parsers::disco::Feature {
    fn from(value: &Feature) -> Self {
        xmpp_parsers::disco::Feature {
            var: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::ns;

    use super::*;

    #[test]
    fn test_ver_hash_exodus() {
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

        assert_eq!(caps.ver_string, "client/pc//Exodus 0.9.1<http://jabber.org/protocol/caps<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/disco#items<http://jabber.org/protocol/muc<");
        assert_eq!(caps.ver_hash, "QgayPKawpkPSDYmwT/WM94uAlu0=");
    }

    #[test]
    fn test_ver_hash_prose() {
        let caps = Capabilities::new(
            "Prose",
            "https://prose.org",
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

        assert_eq!(caps.ver_string, "client/pc//Prose<http://jabber.org/protocol/activity<http://jabber.org/protocol/activity+notify<http://jabber.org/protocol/caps<http://jabber.org/protocol/chatstates<http://jabber.org/protocol/disco#info<http://jabber.org/protocol/pubsub<http://jabber.org/protocol/pubsub#event<http://jabber.org/protocol/pubsub+notify<http://jabber.org/protocol/rsm<jabber:client<jabber:iq:last<jabber:iq:roster<jabber:iq:version<urn:ietf:params:xml:ns:vcard-4.0<urn:ietf:params:xml:ns:vcard-4.0+notify<urn:xmpp:avatar:data<urn:xmpp:avatar:metadata<urn:xmpp:avatar:metadata+notify<urn:xmpp:chat-markers:0<urn:xmpp:delay<urn:xmpp:fallback:0<urn:xmpp:fasten:0<urn:xmpp:hints<urn:xmpp:mam:2<urn:xmpp:message-correct:0<urn:xmpp:message-retract:0<urn:xmpp:ping<urn:xmpp:reactions:0<urn:xmpp:receipts<urn:xmpp:time<");
        assert_eq!(caps.ver_hash, "sRBqzSCojJAWaLc+Y9S2On19bjg=");
    }
}
