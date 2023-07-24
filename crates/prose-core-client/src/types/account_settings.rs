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
