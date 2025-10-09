// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Avatar};
use crate::{ParticipantId, UserId};
use prose_core_client::dtos::{
    JabberClient as CoreJabberClient, ParticipantBasicInfo as CoreParticipantBasicInfo,
    ParticipantInfo as CoreParticipantInfo, RoomAffiliation as CoreRoomAffiliation,
};
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

impl From<CoreParticipantInfo> for ParticipantInfo {
    fn from(value: CoreParticipantInfo) -> Self {
        ParticipantInfo {
            id: value.id.into(),
            user_id: value.user_id.map(Into::into),
            name: value.name,
            is_self: value.is_self,
            availability: value.availability.into(),
            affiliation: value.affiliation.into(),
            avatar: value.avatar.map(|a| Arc::new(a.into())),
            client: value.client.map(Into::into),
            status: value.status,
        }
    }
}

impl From<CoreRoomAffiliation> for RoomAffiliation {
    fn from(value: CoreRoomAffiliation) -> Self {
        match value {
            CoreRoomAffiliation::Outcast => RoomAffiliation::Outcast,
            CoreRoomAffiliation::None => RoomAffiliation::None,
            CoreRoomAffiliation::Member => RoomAffiliation::Member,
            CoreRoomAffiliation::Admin => RoomAffiliation::Admin,
            CoreRoomAffiliation::Owner => RoomAffiliation::Owner,
        }
    }
}

impl From<CoreParticipantBasicInfo> for ParticipantBasicInfo {
    fn from(value: CoreParticipantBasicInfo) -> Self {
        ParticipantBasicInfo {
            id: value.id.into(),
            name: value.name,
            avatar: value.avatar.map(|a| Arc::new(a.into())),
        }
    }
}
