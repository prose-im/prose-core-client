// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::pubsub;

pub trait PubSubItemsExt {
    fn find_first_payload<T: TryFrom<Element>>(
        self,
        name: &str,
        ns: &str,
    ) -> Result<Option<T>, T::Error>;
}

impl PubSubItemsExt for Vec<pubsub::Item> {
    fn find_first_payload<T: TryFrom<Element>>(
        self,
        name: &str,
        ns: &str,
    ) -> Result<Option<T>, T::Error> {
        self.into_iter()
            .find_map(|item| {
                let Some(payload) = &item.payload else {
                    return None;
                };
                if !payload.is(name, ns) {
                    return None;
                }
                Some(T::try_from(payload.clone()))
            })
            .transpose()
    }
}
