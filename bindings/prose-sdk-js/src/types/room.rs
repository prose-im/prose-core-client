// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::IntoJSArray;
use crate::client::WasmError;
use crate::types::{BareJid, BareJidArray, MessagesArray, StringArray};
use alloc::rc::Rc;
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
use prose_core_client::room::{
    DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room as SdkRoom,
};
use prose_core_client::types::{ConnectedRoom, MessageId};
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
    readonly members: BareJID[];

    sendMessage(body: string): Promise<void>;
    updateMessage(messageID: string, body: string): Promise<void>;
    retractMessage(messageID: string): Promise<void>;
    toggleReactionToMessage(id: string, emoji: string): Promise<void>;
    
    loadLatestMessages(since?: string, loadFromServer: boolean): Promise<Message[]>;
    loadMessagesWithIDs(messageIDs: string[]): Promise<Message[]>;
    
    setUserIsComposing(isComposing: boolean): Promise<void>;
    loadComposingUsers(): Promise<BareJID[]>;
    
    saveDraft(message?: string): Promise<void>;
    loadDraft(): Promise<string>;
}

export interface RoomMUC {
    readonly subject?: string;
    
    setSubject(subject?: string): Promise<void>;
}

export interface RoomDirectMessage extends RoomBase {
  type: RoomType.DirectMessage;
}

export interface RoomGroup extends RoomBase, RoomMUC {
  type: RoomType.Group;
}

export interface RoomPrivateChannel extends RoomBase, RoomMUC {
  type: RoomType.PrivateChannel;
}

export interface RoomPublicChannel extends RoomBase, RoomMUC {
  type: RoomType.PublicChannel;
}

export interface RoomGeneric extends RoomBase, RoomMUC {
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

            #[wasm_bindgen(getter)]
            pub fn members(&self) -> BareJidArray {
                self.room
                    .members()
                    .iter()
                    .map(BareJid::from)
                    .collect_into_js_array::<BareJidArray>()
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

            #[wasm_bindgen(js_name = "updateMessage")]
            pub async fn update_message(&self, message_id: &str, body: String) -> Result<()> {
                self.room
                    .update_message(message_id.into(), body)
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }

            #[wasm_bindgen(js_name = "retractMessage")]
            pub async fn retract_message(&self, message_id: &str) -> Result<()> {
                self.room
                    .retract_message(message_id.into())
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }

            #[wasm_bindgen(js_name = "toggleReactionToMessage")]
            pub async fn toggle_reaction_to_message(&self, id: &str, emoji: &str) -> Result<()> {
                self.room
                    .toggle_reaction_to_message(id.into(), emoji.into())
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }

            #[wasm_bindgen(js_name = "loadLatestMessages")]
            pub async fn load_latest_messages(
                &self,
                since: Option<String>,
                load_from_server: bool,
            ) -> Result<MessagesArray> {
                let since: Option<MessageId> = since.map(|id| id.into());

                let messages = self
                    .room
                    .load_latest_messages(since.as_ref(), load_from_server)
                    .await
                    .map_err(WasmError::from)?;

                Ok(messages.into())
            }

            #[wasm_bindgen(js_name = "loadMessagesWithIDs")]
            pub async fn load_messages_with_ids(
                &self,
                message_ids: &StringArray,
            ) -> Result<MessagesArray> {
                let message_ids: Vec<MessageId> = Vec::<String>::try_from(message_ids)?
                    .into_iter()
                    .map(|id| MessageId::from(id))
                    .collect();

                let messages = self
                    .room
                    .load_messages_with_ids(message_ids.as_slice())
                    .await
                    .map_err(WasmError::from)?;

                Ok(messages.into())
            }

            #[wasm_bindgen(js_name = "setUserIsComposing")]
            pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
                self.room
                    .set_user_is_composing(is_composing)
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }

            #[wasm_bindgen(js_name = "saveDraft")]
            pub async fn save_draft(&self, message: Option<String>) -> Result<()> {
                self.room
                    .save_draft(message.as_deref())
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }

            #[wasm_bindgen(js_name = "loadDraft")]
            pub async fn load_draft(&self) -> Result<Option<String>> {
                Ok(self.room.load_draft().await.map_err(WasmError::from)?)
            }
        }
    };
}

macro_rules! muc_room_impl {
    ($t:ident) => {
        #[wasm_bindgen]
        impl $t {
            #[wasm_bindgen(getter)]
            pub fn subject(&self) -> Option<String> {
                self.room.subject()
            }

            #[wasm_bindgen(js_name = "setSubject")]
            pub async fn set_subject(&self, subject: Option<String>) -> Result<()> {
                self.room
                    .set_subject(subject.as_deref())
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

muc_room_impl!(RoomGroup);
muc_room_impl!(RoomPrivateChannel);
muc_room_impl!(RoomPublicChannel);
muc_room_impl!(RoomGeneric);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Room[]")]
    pub type RoomsArray;
}

pub trait ConnectedRoomExt {
    fn into_js_value(self) -> JsValue;
}

impl ConnectedRoomExt for ConnectedRoom<Cache, Cache> {
    fn into_js_value(self) -> JsValue {
        match self {
            ConnectedRoom::DirectMessage(room) => JsValue::from(RoomDirectMessage {
                kind: RoomType::DirectMessage,
                room,
            }),
            ConnectedRoom::Group(room) => JsValue::from(RoomGroup {
                kind: RoomType::Group,
                room,
            }),
            ConnectedRoom::PrivateChannel(room) => JsValue::from(RoomPrivateChannel {
                kind: RoomType::PrivateChannel,
                room,
            }),
            ConnectedRoom::PublicChannel(room) => JsValue::from(RoomPublicChannel {
                kind: RoomType::PublicChannel,
                room,
            }),
            ConnectedRoom::Generic(room) => JsValue::from(RoomGeneric {
                kind: RoomType::Generic,
                room,
            }),
        }
    }
}

impl From<Vec<ConnectedRoom<Cache, Cache>>> for RoomsArray {
    fn from(value: Vec<ConnectedRoom<Cache, Cache>>) -> Self {
        value
            .into_iter()
            .map(|envelope| envelope.into_js_value())
            .collect_into_js_array::<RoomsArray>()
    }
}
