use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::pubsub::Item;

pub struct Publish<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Publish<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("publish").expect("Failed to set name");

        Publish {
            stanza: stanza.into(),
        }
    }

    pub fn node(&self) -> Option<&str> {
        self.attribute("node")
    }

    pub fn set_node(self, node: impl AsRef<str>) -> Self {
        self.set_attribute("node", node)
    }

    pub fn item(&self) -> Option<Item> {
        self.child_by_name("item").map(Into::into)
    }

    pub fn set_item(self, item: Item) -> Self {
        self.add_child(item)
    }
}

stanza_base!(Publish);
