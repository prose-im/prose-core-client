// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::PublicRoomInfo;

use crate::types::BareJid;

#[wasm_bindgen]
pub struct Channel {
    jid: BareJid,
    name: String,
}

#[wasm_bindgen]
impl Channel {
    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.jid.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl From<PublicRoomInfo> for Channel {
    fn from(value: PublicRoomInfo) -> Self {
        Channel {
            jid: value.jid.clone().into_inner().into(),
            name: value
                .name
                .or(value.jid.node_str().map(|n| n.to_string()))
                .unwrap_or(value.jid.to_string()),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Channel[]")]
    pub type ChannelsArray;
}
