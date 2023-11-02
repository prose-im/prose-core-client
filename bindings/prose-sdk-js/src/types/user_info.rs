// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::{
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
            jid: value.jid.into(),
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
            jid: value.jid.into(),
            name: value.name,
            availability: value.availability.into(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "UserBasicInfo[]")]
    pub type UserBasicInfoArray;

    #[wasm_bindgen(typescript_type = "UserPresenceInfo[]")]
    pub type UserPresenceInfoArray;
}
