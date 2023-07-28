use super::Availability;
use crate::types::UserActivity;
use jid::BareJid;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: BareJid,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub status: Option<String>,
    pub groups: Vec<String>,
}
