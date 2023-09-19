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
    readonly type: RoomType;
    readonly id: RoomID;
    readonly name: string;

    sendMessage(body: string): Promise<void>;
}

export interface RoomDirectMessage extends RoomBase {
  type: RoomType.DirectMessage;
}

export interface RoomGroup extends RoomBase {
  type: RoomType.Group;
}

export interface RoomPrivateChannel extends RoomBase {
  type: RoomType.PrivateChannel;
}

export interface RoomPublicChannel extends RoomBase {
  type: RoomType.PublicChannel;
}

export interface RoomGeneric extends RoomBase {
  type: RoomType.Generic;
}

export type Room = RoomDirectMessage | RoomGroup | RoomPrivateChannel | RoomPublicChannel | RoomGeneric;
"#;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum RoomType {
    DirectMessage = 0,
    Group = 1,
    PrivateChannel = 2,
    PublicChannel = 3,
    Generic = 4,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomDirectMessage {
    kind: RoomType,
    room: SdkRoom<DirectMessage, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGroup {
    kind: RoomType,
    room: SdkRoom<Group, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPrivateChannel {
    kind: RoomType,
    room: SdkRoom<PrivateChannel, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPublicChannel {
    kind: RoomType,
    room: SdkRoom<PublicChannel, Cache, Cache>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGeneric {
    kind: RoomType,
    room: SdkRoom<Generic, Cache, Cache>,
}

macro_rules! base_room_impl {
    ($t:ident) => {
        #[wasm_bindgen]
        impl $t {
            #[wasm_bindgen(getter, js_name = "type")]
            pub fn kind(&self) -> RoomType {
                self.kind.clone()
            }

            #[wasm_bindgen(getter)]
            pub fn id(&self) -> String {
                self.room.jid().to_string()
            }

            #[wasm_bindgen(getter)]
            pub fn name(&self) -> String {
                self.room
                    .name()
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
                        kind: RoomType::DirectMessage,
                        room,
                    }),
                    RoomEnvelope::Group(room) => JsValue::from(RoomGroup {
                        kind: RoomType::Group,
                        room,
                    }),
                    RoomEnvelope::PrivateChannel(room) => JsValue::from(RoomPrivateChannel {
                        kind: RoomType::PrivateChannel,
                        room,
                    }),
                    RoomEnvelope::PublicChannel(room) => JsValue::from(RoomPublicChannel {
                        kind: RoomType::PublicChannel,
                        room,
                    }),
                    RoomEnvelope::Generic(room) => JsValue::from(RoomGeneric {
                        kind: RoomType::Generic,
                        room,
                    }),
                }
            })
            .collect_into_js_array::<RoomsArray>()
    }
}
