// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::MamVersion;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomFeatures {
    /// Which MAM version does the room support, if any?
    pub mam_version: Option<MamVersion>,
}

impl RoomFeatures {
    pub fn is_mam_supported(&self) -> bool {
        self.mam_version.is_some()
    }
}
