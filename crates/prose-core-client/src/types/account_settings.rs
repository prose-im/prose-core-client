// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::Availability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountSettings {
    pub availability: Availability,
}

impl Default for AccountSettings {
    fn default() -> Self {
        AccountSettings {
            availability: Availability::Available,
        }
    }
}
