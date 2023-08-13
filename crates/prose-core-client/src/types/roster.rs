// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use xmpp_parsers::roster;

#[derive(Debug, PartialEq, Display, EnumString, Clone, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
pub enum Subscription {
    None,
    From,
    To,
    Both,
    Remove,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Item {
    pub jid: BareJid,
    pub subscription: Subscription,
    pub groups: Vec<String>,
}

impl From<roster::Item> for Item {
    fn from(value: roster::Item) -> Self {
        Item {
            jid: value.jid,
            subscription: value.subscription.into(),
            groups: value.groups.into_iter().map(|g| g.0).collect(),
        }
    }
}

impl From<roster::Subscription> for Subscription {
    fn from(value: roster::Subscription) -> Self {
        match value {
            roster::Subscription::None => Subscription::None,
            roster::Subscription::From => Subscription::From,
            roster::Subscription::To => Subscription::To,
            roster::Subscription::Both => Subscription::Both,
            roster::Subscription::Remove => Subscription::Remove,
        }
    }
}
