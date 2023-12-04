// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::UserResourceId;

use super::ServerFeatures;

pub struct ConnectionProperties {
    /// The JID of our connected user.
    pub connected_jid: UserResourceId,
    /// The features of the server we're connected with.
    pub server_features: ServerFeatures,
}
