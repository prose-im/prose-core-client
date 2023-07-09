use serde::{Deserialize, Serialize};

use prose_core_domain::Availability;

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
