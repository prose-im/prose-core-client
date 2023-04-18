use jid::Jid;

use crate::helpers::id_string_macro::id_string;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::pubsub::subscription::Kind;

id_string!(Id);

pub struct Subscription<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Subscription<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("subscription").expect("Failed to set name");

        Subscription {
            stanza: stanza.into(),
        }
    }

    pub fn node(&self) -> Option<&str> {
        self.attribute("node")
    }

    pub fn set_node(self, node: impl AsRef<str>) -> Self {
        self.set_attribute("node", node)
    }

    fn jid(&self) -> Option<Jid> {
        self.stanza()
            .get_attribute("jid")
            .and_then(|s| s.parse::<Jid>().ok())
    }

    fn set_jid(self, from: impl Into<Jid>) -> Self {
        self.set_attribute("from", from.into().to_string())
    }

    pub fn subid(&self) -> Option<Id> {
        self.attribute("subid").map(|s| s.into())
    }

    pub fn set_subid(self, id: Id) -> Self {
        self.set_attribute("subid", id.as_ref())
    }

    pub fn subscription(&self) -> Option<Kind> {
        self.stanza
            .get_attribute("subscription")
            .and_then(|s| s.parse::<Kind>().ok())
    }

    pub fn set_subscription(self, kind: Kind) -> Self {
        self.set_attribute("subscription", kind.to_string())
    }
}

stanza_base!(Subscription);
