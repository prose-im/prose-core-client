use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::pubsub::subscribe::Subscribe;
use crate::stanza::pubsub::{Items, Publish, Retract};
use crate::stanza::Namespace;

pub struct PubSub<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> PubSub<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("pubsub").expect("Failed to set name");
        stanza.set_ns(Namespace::PubSub.to_string()).unwrap();

        PubSub {
            stanza: stanza.into(),
        }
    }

    pub fn items(&self) -> Option<Items> {
        self.child_by_name("items").map(Into::into)
    }

    pub fn set_items(self, items: Items) -> Self {
        self.add_child(items)
    }

    pub fn publish(&self) -> Option<Publish> {
        self.child_by_name("publish").map(Into::into)
    }

    pub fn set_publish(self, publish: Publish) -> Self {
        self.add_child(publish)
    }

    pub fn subscribe(&self) -> Option<Subscribe> {
        self.child_by_name("subscribe").map(Into::into)
    }

    pub fn set_subscribe(self, subscribe: Subscribe) -> Self {
        self.add_child(subscribe)
    }

    pub fn retract(&self) -> Option<Retract> {
        self.child_by_name("retract").map(Into::into)
    }

    pub fn set_retract(self, retract: Retract) -> Self {
        self.add_child(retract)
    }
}

stanza_base!(PubSub);
