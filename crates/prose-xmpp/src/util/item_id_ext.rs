// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::pubsub::ItemId;

pub trait ItemIdExt {
    fn current() -> Self;
}

impl ItemIdExt for ItemId {
    /// https://xmpp.org/extensions/xep-0060.html#impl-singleton
    fn current() -> Self {
        ItemId("current".to_string())
    }
}
