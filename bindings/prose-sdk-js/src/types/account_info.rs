// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::*;

use prose_core_client::dtos::AccountInfo as CoreAccountInfo;

use super::{contact::UserStatus, Availability, BareJid};

#[wasm_bindgen]
pub struct AccountInfo(CoreAccountInfo);

impl From<CoreAccountInfo> for AccountInfo {
    fn from(value: CoreAccountInfo) -> Self {
        AccountInfo(value)
    }
}

#[wasm_bindgen]
impl AccountInfo {
    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.id.clone().into_inner().into()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.0.availability.into()
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<UserStatus> {
        self.0
            .status
            .as_ref()
            .map(|activity| activity.clone().into())
    }
}
