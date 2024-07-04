// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::UserId;

#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub id: UserId,
    pub name: Option<String>,
    pub presence_subscription: PresenceSubscription,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PresenceSubscription {
    // We have requested to subscribe to the contact's presence, but they haven't approved yet.
    Requested,

    // Both we and the contact are subscribed to each other's presence.
    Mutual,

    // The contact is subscribed to our presence, so they can see our status.
    TheyFollow,

    // We are subscribed to the contact's presence, so we can see their status.
    WeFollow,

    // There is no presence subscription between us and the contact.
    None,
}
