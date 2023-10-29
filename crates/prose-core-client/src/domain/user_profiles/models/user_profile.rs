// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::shared::utils::concatenate_names;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub org: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub tel: Option<String>,
    pub url: Option<Url>,
    pub address: Option<Address>,
}

impl UserProfile {
    pub fn full_name(&self) -> Option<String> {
        concatenate_names(&self.first_name, &self.last_name)
    }
}
