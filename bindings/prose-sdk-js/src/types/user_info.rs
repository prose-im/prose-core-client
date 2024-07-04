// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::{
    Avatar as SdkAvatar, JabberClient as SdkJabberClient,
    ParticipantBasicInfo as SdkParticipantBasicInfo, ParticipantInfo as SdkParticipantInfo,
    RoomAffiliation as SdkRoomAffiliation, UserBasicInfo as SdkUserBasicInfo,
    UserPresenceInfo as SdkUserPresenceInfo,
};

use crate::types::{Availability, BareJid, ParticipantId, UserStatus};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Avatar(SdkAvatar);

#[wasm_bindgen]
#[derive(Clone)]
pub struct JabberClient(SdkJabberClient);

#[wasm_bindgen]
#[derive(Clone)]
pub enum RoomAffiliation {
    Outcast = 0,
    None = 1,
    Member = 2,
    Admin = 3,
    Owner = 4,
}

#[wasm_bindgen]
pub struct UserBasicInfo(SdkUserBasicInfo);

#[wasm_bindgen]
pub struct UserPresenceInfo(SdkUserPresenceInfo);

#[wasm_bindgen]
pub struct ParticipantBasicInfo(SdkParticipantBasicInfo);

#[wasm_bindgen]
pub struct ParticipantInfo(SdkParticipantInfo);

#[wasm_bindgen]
impl JabberClient {
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen(js_name = "isProse")]
    pub fn is_prose(&self) -> bool {
        self.0.is_prose()
    }
}

#[wasm_bindgen]
impl UserBasicInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.id.clone().into_inner().into()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar(&self) -> Option<Avatar> {
        self.0.avatar.clone().map(Into::into)
    }
}

#[wasm_bindgen]
impl UserPresenceInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.id.clone().into_inner().into()
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.0.availability.into()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar(&self) -> Option<Avatar> {
        self.0.avatar.clone().map(Into::into)
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<UserStatus> {
        self.0.status.clone().map(Into::into)
    }
}

#[wasm_bindgen]
impl ParticipantBasicInfo {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> ParticipantId {
        self.0.id.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar(&self) -> Option<Avatar> {
        self.0.avatar.clone().map(Into::into)
    }
}

#[wasm_bindgen]
impl ParticipantInfo {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> ParticipantId {
        self.0.id.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> Option<BareJid> {
        self.0.user_id.clone().map(|id| id.into_inner().into())
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter, js_name = "isSelf")]
    pub fn is_self(&self) -> bool {
        self.0.is_self
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.0.availability.into()
    }

    #[wasm_bindgen(getter)]
    pub fn affiliation(&self) -> RoomAffiliation {
        self.0.affiliation.into()
    }

    #[wasm_bindgen(getter)]
    pub fn avatar(&self) -> Option<Avatar> {
        self.0.avatar.clone().map(Into::into)
    }

    #[wasm_bindgen(getter)]
    pub fn client(&self) -> Option<JabberClient> {
        self.0.client.clone().map(Into::into)
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<String> {
        self.0.status.clone()
    }
}

impl From<SdkAvatar> for Avatar {
    fn from(value: SdkAvatar) -> Self {
        Self(value)
    }
}

impl From<SdkJabberClient> for JabberClient {
    fn from(value: SdkJabberClient) -> Self {
        Self(value)
    }
}

impl From<Avatar> for SdkAvatar {
    fn from(value: Avatar) -> Self {
        value.0
    }
}

impl From<SdkUserBasicInfo> for UserBasicInfo {
    fn from(value: SdkUserBasicInfo) -> Self {
        Self(value)
    }
}

impl From<SdkUserPresenceInfo> for UserPresenceInfo {
    fn from(value: SdkUserPresenceInfo) -> Self {
        Self(value)
    }
}

impl From<SdkParticipantBasicInfo> for ParticipantBasicInfo {
    fn from(value: SdkParticipantBasicInfo) -> Self {
        Self(value)
    }
}

impl From<SdkParticipantInfo> for ParticipantInfo {
    fn from(value: SdkParticipantInfo) -> Self {
        Self(value)
    }
}

impl From<SdkRoomAffiliation> for RoomAffiliation {
    fn from(value: SdkRoomAffiliation) -> Self {
        match value {
            SdkRoomAffiliation::Outcast => RoomAffiliation::Outcast,
            SdkRoomAffiliation::None => RoomAffiliation::None,
            SdkRoomAffiliation::Member => RoomAffiliation::Member,
            SdkRoomAffiliation::Admin => RoomAffiliation::Admin,
            SdkRoomAffiliation::Owner => RoomAffiliation::Owner,
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "UserBasicInfo[]")]
    pub type UserBasicInfoArray;

    #[wasm_bindgen(typescript_type = "UserPresenceInfo[]")]
    pub type UserPresenceInfoArray;

    #[wasm_bindgen(typescript_type = "ParticipantBasicInfo[]")]
    pub type ParticipantBasicInfoArray;

    #[wasm_bindgen(typescript_type = "ParticipantInfo[]")]
    pub type ParticipantInfoArray;
}
