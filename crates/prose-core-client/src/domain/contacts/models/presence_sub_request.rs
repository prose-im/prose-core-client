// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct PresenceSubRequest {
    /// The id of the user that wants to subscribe to our presence.
    pub user_id: UserId,
    /// The nickname of the user that wants to subscribe to our presence.
    /// https://xmpp.org/extensions/xep-0172.html#example-3
    pub name: Option<String>,
}
