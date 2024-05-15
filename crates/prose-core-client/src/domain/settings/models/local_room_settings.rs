// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::messaging::models::MessageRef;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LocalRoomSettings {
    pub last_catchup_time: Option<DateTime<Utc>>,
    pub last_read_message: Option<MessageRef>,
}
