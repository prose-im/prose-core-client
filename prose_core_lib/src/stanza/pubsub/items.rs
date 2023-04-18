use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::pubsub::Item;

pub struct Items<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Items<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("items").unwrap();

        Items {
            stanza: stanza.into(),
        }
    }

    pub fn node(&self) -> Option<&str> {
        self.attribute("node")
    }

    pub fn set_node(self, node: impl AsRef<str>) -> Self {
        self.set_attribute("node", node)
    }

    pub fn max_items(&self) -> Option<i64> {
        self.attribute("max_items")
            .and_then(|str| i64::from_str_radix(str, 10).ok())
    }

    pub fn set_max_items(self, max_items: i64) -> Self {
        self.set_attribute("max_items", max_items.to_string())
    }

    pub fn items(&self) -> impl Iterator<Item = Item> {
        self.children().filter_map(|child| {
            if child.name() != Some("item") {
                return None;
            }
            return Some(child.into());
        })
    }

    pub fn set_items<'b>(self, items: impl IntoIterator<Item = Item<'b>>) -> Self {
        self.add_children(items)
    }
}

stanza_base!(Items);
