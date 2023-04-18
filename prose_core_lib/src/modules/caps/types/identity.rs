use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;

pub struct Identity<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Identity<'a> {
    pub fn new<'b>(
        category: impl AsRef<str>,
        kind: impl AsRef<str>,
        name: impl Into<Option<&'b str>>,
    ) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("identity").expect("Failed to set name");
        stanza
            .set_attribute("category", category)
            .expect("Failed to set attribute");
        stanza
            .set_attribute("type", kind)
            .expect("Failed to set attribute");

        if let Some(name) = name.into() {
            stanza
                .set_attribute("name", name)
                .expect("Failed to set attribute");
        }

        Identity {
            stanza: stanza.into(),
        }
    }
}

impl<'a> Identity<'a> {
    pub fn category(&self) -> Option<&str> {
        self.attribute("category")
    }

    pub fn kind(&self) -> Option<&str> {
        self.attribute("type")
    }

    pub fn name(&self) -> Option<&str> {
        self.attribute("name")
    }
}

stanza_base!(Identity);
