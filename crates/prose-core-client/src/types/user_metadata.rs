use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    /// XEP-0202: Entity Time
    pub local_time: Option<DateTime<FixedOffset>>,
    /// XEP-0012: Last Activity
    pub last_activity: Option<LastActivity>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LastActivity {
    /// The time when the user was last active
    pub timestamp: DateTime<Utc>,
    /// The status they sent when setting their presence to 'unavailable'.
    pub status: Option<String>,
}
