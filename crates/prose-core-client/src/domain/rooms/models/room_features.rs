// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeDelta, Utc};

use crate::domain::shared::models::MamVersion;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomFeatures {
    /// Which MAM version does the room support, if any?
    pub mam_version: Option<MamVersion>,
    pub server_time_offset: TimeDelta,
    /// Does the server support XEP-0410 (MUC Self-Ping)?
    pub self_ping_optimization: bool,
}

impl RoomFeatures {
    pub fn is_mam_supported(&self) -> bool {
        self.mam_version.is_some()
    }

    pub fn local_time_to_server_time(&self, local_time: DateTime<Utc>) -> DateTime<Utc> {
        local_time + self.server_time_offset
    }
}
