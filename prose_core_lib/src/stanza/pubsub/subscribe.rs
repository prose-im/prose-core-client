use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use jid::Jid;

pub struct Subscribe<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Subscribe<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("subscribe").expect("Failed to set name");

        Subscribe {
            stanza: stanza.into(),
        }
    }

    pub fn node(&self) -> Option<&str> {
        self.attribute("node")
    }

    pub fn set_node(self, node: impl AsRef<str>) -> Self {
        self.set_attribute("node", node)
    }

    pub fn jid(&self) -> Option<Jid> {
        self.stanza()
            .get_attribute("jid")
            .and_then(|s| s.parse::<Jid>().ok())
    }

    pub fn set_jid(self, from: impl Into<Jid>) -> Self {
        self.set_attribute("jid", from.into().to_string())
    }
}

stanza_base!(Subscribe);
