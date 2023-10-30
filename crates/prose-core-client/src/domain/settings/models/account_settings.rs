// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::Availability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct AccountSettings {
    /// The last configured availability
    pub availability: Option<Availability>,
    /// The generated resource string use to form a FullJid
    pub resource: Option<String>,
}
