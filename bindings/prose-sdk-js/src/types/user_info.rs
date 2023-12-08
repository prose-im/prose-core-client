// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::{
    ParticipantInfo as SdkParticipantInfo, RoomAffiliation as SdkRoomAffiliation,
    UserBasicInfo as SdkUserBasicInfo, UserPresenceInfo as SdkUserPresenceInfo,
};

use crate::types::{Availability, BareJid};

#[wasm_bindgen]
pub struct UserBasicInfo {
    jid: BareJid,
    name: String,
}

#[wasm_bindgen]
impl UserBasicInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.jid.clone().into()
    }
}

impl From<SdkUserBasicInfo> for UserBasicInfo {
    fn from(value: SdkUserBasicInfo) -> Self {
        Self {
            jid: value.id.into_inner().into(),
            name: value.name,
        }
    }
}

#[wasm_bindgen]
pub struct UserPresenceInfo {
    jid: BareJid,
    name: String,
    availability: Availability,
}

#[wasm_bindgen]
impl UserPresenceInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.jid.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.availability.clone()
    }
}

impl From<SdkUserPresenceInfo> for UserPresenceInfo {
    fn from(value: SdkUserPresenceInfo) -> Self {
        Self {
            jid: value.id.into_inner().into(),
            name: value.name,
            availability: value.availability.into(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum RoomAffiliation {
    Outcast = 0,
    None = 1,
    Member = 2,
    Admin = 3,
    Owner = 4,
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
pub struct ParticipantInfo {
    jid: Option<BareJid>,
    name: String,
    availability: Availability,
    affiliation: RoomAffiliation,
}

#[wasm_bindgen]
impl ParticipantInfo {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> Option<BareJid> {
        self.jid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.availability.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn affiliation(&self) -> RoomAffiliation {
        self.affiliation.clone()
    }
}

impl From<SdkParticipantInfo> for ParticipantInfo {
    fn from(value: SdkParticipantInfo) -> Self {
        Self {
            jid: value.id.map(|id| id.into_inner().into()),
            name: value.name,
            availability: value.availability.into(),
            affiliation: value.affiliation.into(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "UserBasicInfo[]")]
    pub type UserBasicInfoArray;

    #[wasm_bindgen(typescript_type = "UserPresenceInfo[]")]
    pub type UserPresenceInfoArray;

    #[wasm_bindgen(typescript_type = "ParticipantInfo[]")]
    pub type ParticipantInfoArray;
}
