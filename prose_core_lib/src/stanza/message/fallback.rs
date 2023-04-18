use crate::helpers::StanzaCow;
use crate::stanza::Namespace;
use crate::stanza_base;

/// XEP-0428
pub struct Fallback<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Fallback<'a> {
    pub fn new(r#for: Option<Namespace>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("fallback").expect("Failed to set name");
        stanza
            .set_ns(Namespace::Fallback.to_string())
            .expect("Failed to set namespace");
        if let Some(ns) = r#for {
            stanza
                .set_attribute("for", ns.to_string())
                .expect("Failed to set attribute");
        }

        Fallback {
            stanza: stanza.into(),
        }
    }

    pub fn r#for(&self) -> Option<Namespace> {
        self.attribute("for")
            .and_then(|s| s.parse::<Namespace>().ok())
    }
}

impl<'a> Fallback<'a> {}

stanza_base!(Fallback);
