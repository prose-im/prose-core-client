// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub;
use xmpp_parsers::pubsub::pubsub::Items;
use xmpp_parsers::pubsub::{Item, ItemId, NodeName, SubscriptionId};

pub struct PubSubQuery {
    id: String,
    to: Option<BareJid>,
    items: Items,
}

impl PubSubQuery {
    pub fn new(id: impl Into<String>, node: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            to: None,
            items: Items {
                max_items: None,
                node: NodeName(node.into()),
                subid: None,
                items: vec![],
            },
        }
    }

    pub fn set_to(mut self, to: BareJid) -> Self {
        self.to = Some(to);
        self
    }

    pub fn set_sub_id(mut self, sub_id: impl Into<String>) -> Self {
        self.items.subid = Some(SubscriptionId(sub_id.into()));
        self
    }

    pub fn set_item_ids<Id: Into<String>>(mut self, ids: impl IntoIterator<Item = Id>) -> Self {
        self.items.items = ids
            .into_iter()
            .map(|id| {
                pubsub::pubsub::Item(Item {
                    id: Some(ItemId(id.into())),
                    publisher: None,
                    payload: None,
                })
            })
            .collect();
        self
    }

    pub fn set_max_items(mut self, max_items: u32) -> Self {
        self.items.max_items = Some(max_items);
        self
    }

    pub fn build(self) -> Iq {
        let mut iq = Iq::from_get(self.id, pubsub::PubSub::Items(self.items));
        if let Some(to) = self.to {
            iq.to = Some(to.into());
        }
        iq
    }
}
