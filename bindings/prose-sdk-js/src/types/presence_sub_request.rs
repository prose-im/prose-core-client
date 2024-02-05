// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use crate::types::BareJid;
use prose_core_client::dtos::{
    PresenceSubRequest as CorePresenceSubRequest, PresenceSubRequestId as CorePresenceSubRequestId,
};

#[wasm_bindgen]
pub struct PresenceSubRequestId(CorePresenceSubRequestId);

#[wasm_bindgen]
pub struct PresenceSubRequest(CorePresenceSubRequest);

#[wasm_bindgen]
impl PresenceSubRequest {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> PresenceSubRequestId {
        PresenceSubRequestId(self.0.id.clone())
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.user_id.clone().into_inner().into()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "PresenceSubRequest[]")]
    pub type PresenceSubRequestArray;
}

impl From<CorePresenceSubRequest> for PresenceSubRequest {
    fn from(value: CorePresenceSubRequest) -> Self {
        PresenceSubRequest(value)
    }
}

impl AsRef<CorePresenceSubRequestId> for PresenceSubRequestId {
    fn as_ref(&self) -> &CorePresenceSubRequestId {
        &self.0
    }
}
