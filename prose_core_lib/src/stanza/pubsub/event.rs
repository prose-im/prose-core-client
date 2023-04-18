use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::pubsub::Items;
use crate::stanza::Namespace;

pub struct Event<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Event<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("pubsub").expect("Failed to set name");
        stanza.set_ns(Namespace::PubSub.to_string()).unwrap();

        Event {
            stanza: stanza.into(),
        }
    }

    pub fn items(&self) -> Option<Items> {
        self.child_by_name("items").map(Into::into)
    }

    pub fn items_with_namespace(&self, ns: Namespace) -> Option<Items> {
        self.child_by_name_and_namespace("items", ns)
            .map(Into::into)
    }

    pub fn set_items(self, items: Items) -> Self {
        self.add_child(items)
    }
}

stanza_base!(Event);
