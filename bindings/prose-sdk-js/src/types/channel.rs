use crate::types::BareJid;
use prose_xmpp::mods;
use wasm_bindgen::prelude::wasm_bindgen;

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

impl From<mods::muc::Room> for Channel {
    fn from(value: mods::muc::Room) -> Self {
        Channel {
            jid: value.jid.to_bare().into(),
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
