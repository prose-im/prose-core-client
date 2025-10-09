// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{DateTime, DateTimeFixed};
use prose_core_client::dtos::{LastActivity as CoreLastActivity, UserMetadata as CoreUserMetadata};

#[derive(uniffi::Record)]
pub struct UserMetadata {
    pub local_time: Option<DateTimeFixed>,
    pub last_activity: Option<LastActivity>,
}

#[derive(uniffi::Record)]
pub struct LastActivity {
    pub timestamp: DateTime,
    pub status: Option<String>,
}

impl From<CoreUserMetadata> for UserMetadata {
    fn from(value: CoreUserMetadata) -> Self {
        UserMetadata {
            local_time: value.local_time.map(Into::into),
            last_activity: value.last_activity.map(Into::into),
        }
    }
}

impl From<CoreLastActivity> for LastActivity {
    fn from(value: CoreLastActivity) -> Self {
        LastActivity {
            timestamp: value.timestamp.into(),
            status: value.status,
        }
    }
}
