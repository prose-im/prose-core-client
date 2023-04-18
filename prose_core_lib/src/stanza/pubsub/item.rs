use crate::helpers::id_string_macro::id_string;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;

id_string!(Id);

pub struct Item<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Item<'a> {
    pub fn new(id: impl Into<Id>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("item").unwrap();
        stanza.set_id(id.into().as_ref()).unwrap();

        Item {
            stanza: stanza.into(),
        }
    }

    pub fn id(&self) -> Option<Id> {
        self.stanza.get_attribute("id").map(|s| s.into())
    }
}

stanza_base!(Item);
