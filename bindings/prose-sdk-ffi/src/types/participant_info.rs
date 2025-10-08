// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Avatar};
use crate::{ParticipantId, UserId};
use prose_core_client::dtos::JabberClient as CoreJabberClient;
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct ParticipantInfo {
    pub id: ParticipantId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub is_self: bool,
    pub availability: Availability,
    pub affiliation: RoomAffiliation,
    pub avatar: Option<Arc<Avatar>>,
    pub client: Option<JabberClient>,
    pub status: Option<String>,
}

#[derive(uniffi::Record)]
pub struct ParticipantBasicInfo {
    pub id: ParticipantId,
    pub name: String,
    pub avatar: Option<Arc<Avatar>>,
}

#[derive(uniffi::Enum)]
pub enum RoomAffiliation {
    Outcast,
    None,
    Member,
    Admin,
    Owner,
}

#[derive(uniffi::Record)]
pub struct JabberClient {
    name: String,
    is_prose: bool,
}

impl From<CoreJabberClient> for JabberClient {
    fn from(client: CoreJabberClient) -> Self {
        JabberClient {
            name: client.to_string(),
            is_prose: client.is_prose(),
        }
    }
}
