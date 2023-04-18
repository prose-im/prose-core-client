use crate::helpers::StanzaCow;
use crate::modules::profile::types::avatar::Info;
use crate::stanza::Namespace;
use crate::stanza_base;

pub struct Metadata<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Metadata<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("metadata").expect("Failed to set name");
        stanza
            .set_ns(Namespace::AvatarMetadata.to_string())
            .expect("Failed to set namespace");

        Metadata {
            stanza: stanza.into(),
        }
    }

    pub fn info(&self) -> Option<Info> {
        self.child_by_name("info").map(Into::into)
    }

    pub fn set_info(self, info: Info) -> Self {
        self.add_child(info)
    }
}

stanza_base!(Metadata);
