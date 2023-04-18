use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;

pub struct Feature<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Feature<'a> {
    pub fn new(var: impl AsRef<str>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("feature").expect("Failed to set name");
        stanza
            .set_attribute("var", var)
            .expect("Failed to set attribute");

        Feature {
            stanza: stanza.into(),
        }
    }
}

impl<'a> Feature<'a> {
    pub fn var(&self) -> Option<&str> {
        self.attribute("var")
    }
}

stanza_base!(Feature);
