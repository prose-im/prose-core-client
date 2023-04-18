use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::Namespace;

pub struct Caps<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Caps<'a> {
    pub fn new(hash: impl AsRef<str>, node: impl AsRef<str>, ver: impl AsRef<str>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("c").expect("Failed to set name");
        stanza
            .set_ns(Namespace::Caps.to_string())
            .expect("Failed to set namespace");
        stanza
            .set_attribute("hash", hash)
            .expect("Failed to set attribute");
        stanza
            .set_attribute("node", node)
            .expect("Failed to set attribute");
        stanza
            .set_attribute("ver", ver)
            .expect("Failed to set attribute");

        Caps {
            stanza: stanza.into(),
        }
    }
}

impl<'a> Caps<'a> {
    pub fn hash(&self) -> Option<&str> {
        self.attribute("hash")
    }

    pub fn set_hash(self, hash: impl AsRef<str>) -> Self {
        self.set_attribute("hash", hash)
    }

    pub fn node(&self) -> Option<&str> {
        self.attribute("node")
    }

    pub fn set_node(self, node: impl AsRef<str>) -> Self {
        self.set_attribute("node", node)
    }

    pub fn ver(&self) -> Option<&str> {
        self.attribute("ver")
    }

    pub fn set_ver(self, ver: impl AsRef<str>) -> Self {
        self.set_attribute("ver", ver)
    }
}

stanza_base!(Caps);
