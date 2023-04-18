use crate::helpers::StanzaCow;
use crate::stanza_base;

pub struct Address<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Address<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("adr").expect("Failed to set name");

        Address {
            stanza: stanza.into(),
        }
    }

    pub fn locality(&self) -> Option<String> {
        self.child_by_name("locality")?.text()
    }

    pub fn set_locality(self, locality: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("locality", locality))
    }

    pub fn country(&self) -> Option<String> {
        self.child_by_name("country")?.text()
    }

    pub fn set_country(self, country: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("country", country))
    }
}

stanza_base!(Address);
