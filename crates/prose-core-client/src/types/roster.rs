// prose-core-client/prose-core-client
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Group {
    Favorite,
    Team,
    Other,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Item {
    pub jid: BareJid,
    pub name: Option<String>,
    pub subscription: Subscription,
    pub group: Group,
    pub is_me: bool,
}

impl From<(&BareJid, roster::Item)> for Item {
    fn from(value: (&BareJid, roster::Item)) -> Self {
        let group = value
            .1
            .groups
            .first()
            .map(|group| {
                if group.0 == Group::Favorite.to_string() {
                    return Group::Favorite;
                }
                if value.1.jid.domain() == value.0.domain() {
                    return Group::Team;
                }
                Group::Other
            })
            .unwrap_or(Group::Other);

        Item {
            jid: value.1.jid,
            name: value.1.name,
            subscription: value.1.subscription.into(),
            group,
            is_me: false,
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
