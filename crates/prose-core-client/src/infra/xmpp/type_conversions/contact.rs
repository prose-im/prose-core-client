// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::roster::{Ask, Item, Subscription};

use crate::domain::contacts::models::{Contact, PresenceSubscription};

impl From<Item> for Contact {
    fn from(roster_item: Item) -> Self {
        let presence_subscription = PresenceSubscription::from(&roster_item);

        Contact {
            id: roster_item.jid.into(),
            presence_subscription,
        }
    }
}

impl From<&Item> for PresenceSubscription {
    fn from(value: &Item) -> Self {
        match (&value.subscription, &value.ask) {
            (Subscription::None, Ask::Subscribe) => PresenceSubscription::Requested,
            (Subscription::None, Ask::None) => PresenceSubscription::None,
            (Subscription::Both, _) => PresenceSubscription::Mutual,
            (Subscription::From, _) => PresenceSubscription::TheyFollow,
            (Subscription::To, _) => PresenceSubscription::WeFollow,
            (Subscription::Remove, _) => PresenceSubscription::None,
        }
    }
}
