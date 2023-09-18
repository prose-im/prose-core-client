// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::IntoJSArray;
use crate::client::WasmError;
use alloc::rc::Rc;
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
use prose_core_client::room::{
    DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room as SdkRoom, RoomEnvelope,
};
use tracing::info;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

type Cache = Rc<IndexedDBDataCache>;
type Result<T, E = JsError> = std::result::Result<T, E>;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type RoomID = string;

export interface RoomBase {
    readonly id: RoomID;
    readonly name: string;

    sendMessage(body: string): Promise<void>;
}

export interface RoomDirectMessage extends RoomBase {
  kind: "direct-message";
}

export interface RoomGroup extends RoomBase {
  kind: "group";
}

export interface RoomPrivateChannel extends RoomBase {
  kind: "private-channel";
}

export interface RoomPublicChannel extends RoomBase {
  kind: "public-channel";
}

export interface RoomGeneric extends RoomBase {
  kind: "generic";
}

export type Room = RoomDirectMessage | RoomGroup | RoomPrivateChannel | RoomPublicChannel;
"#;

#[wasm_bindgen(skip_typescript)]
pub struct RoomDirectMessage {
    kind: String,
    room: SdkRoom<DirectMessage, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGroup {
    kind: String,
    room: SdkRoom<Group, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPrivateChannel {
    kind: String,
    room: SdkRoom<PrivateChannel, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPublicChannel {
    kind: String,
    room: SdkRoom<PublicChannel, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGeneric {
    kind: String,
    room: SdkRoom<Generic, Cache, Cache>,
}

macro_rules! base_room_impl {
    ($t:ident) => {
        #[wasm_bindgen]
        impl $t {
            #[wasm_bindgen(getter)]
            pub fn kind(&self) -> String {
                self.kind.clone()
            }

            #[wasm_bindgen(getter)]
            pub fn id(&self) -> String {
                self.room.jid.to_string()
            }

            #[wasm_bindgen(getter)]
            pub fn name(&self) -> String {
                self.room
                    .name
                    .as_deref()
                    .unwrap_or("<untitled>")
                    .to_string()
            }

            #[wasm_bindgen(js_name = "sendMessage")]
            pub async fn send_message(&self, body: String) -> Result<()> {
                info!("Sending message…");
                self.room
                    .send_message(body)
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }
        }
    };
}

base_room_impl!(RoomDirectMessage);
base_room_impl!(RoomGroup);
base_room_impl!(RoomPrivateChannel);
base_room_impl!(RoomPublicChannel);
base_room_impl!(RoomGeneric);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Room[]")]
    pub type RoomsArray;
}

impl From<Vec<RoomEnvelope<Cache, Cache>>> for RoomsArray {
    fn from(value: Vec<RoomEnvelope<Cache, Cache>>) -> Self {
        value
            .into_iter()
            .map(|envelope| -> JsValue {
                match envelope {
                    RoomEnvelope::DirectMessage(room) => JsValue::from(RoomDirectMessage {
                        kind: "direct-message".to_string(),
                        room,
                    }),
                    RoomEnvelope::Group(room) => JsValue::from(RoomGroup {
                        kind: "group".to_string(),
                        room,
                    }),
                    RoomEnvelope::PrivateChannel(room) => JsValue::from(RoomPrivateChannel {
                        kind: "private-channel".to_string(),
                        room,
                    }),
                    RoomEnvelope::PublicChannel(room) => JsValue::from(RoomPublicChannel {
                        kind: "public-channel".to_string(),
                        room,
                    }),
                    RoomEnvelope::Generic(room) => JsValue::from(RoomGeneric {
                        kind: "generic".to_string(),
                        room,
                    }),
                }
            })
            .collect_into_js_array::<RoomsArray>()
    }
}
