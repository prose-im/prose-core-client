// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use js_sys::Array;
use tracing::info;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

use prose_core_client::dtos::MessageId;
use prose_core_client::services::{
    DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room as SdkRoom, RoomEnvelope,
};

use crate::client::WasmError;
use crate::types::{
    try_jid_vec_from_string_array, BareJid, BareJidArray, MessagesArray, StringArray,
};

use super::IntoJSArray;

type Result<T, E = JsError> = std::result::Result<T, E>;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type RoomID = string;

export interface RoomBase {
    readonly type: RoomType;
    readonly id: RoomID;
    readonly name: string;
    /// The members of a room. Only available for DirectMessage and Group (member-only rooms)
    readonly members: JID[];
    /// The occupants of a room.
    readonly occupants: JID[];

    sendMessage(body: string): Promise<void>;
    updateMessage(messageID: string, body: string): Promise<void>;
    retractMessage(messageID: string): Promise<void>;
    toggleReactionToMessage(id: string, emoji: string): Promise<void>;
    
    loadLatestMessages(since?: string, loadFromServer: boolean): Promise<Message[]>;
    loadMessagesWithIDs(messageIDs: string[]): Promise<Message[]>;
    
    setUserIsComposing(isComposing: boolean): Promise<void>;
    loadComposingUsers(): Promise<JID[]>;
    
    saveDraft(message?: string): Promise<void>;
    loadDraft(): Promise<string>;
}

export interface RoomMUC {
    readonly subject?: string;
    
    setTopic(topic?: string): Promise<void>;
}

export interface RoomChannel {
    /// Pass an array of valid BareJid strings.
    inviteUsers(users: string[]): Promise<void>;
}

export interface RoomDirectMessage extends RoomBase {
  type: RoomType.DirectMessage;
}

export interface RoomGroup extends RoomBase, RoomMUC {
  type: RoomType.Group;
  
  /// Resends invites to its members. Can be useful if someone accidentally rejected the invite or 
  /// deleted their bookmark.
  resendInvitesToMembers(): Promise<void>;
}

export interface RoomPrivateChannel extends RoomBase, RoomMUC, RoomChannel {
  type: RoomType.PrivateChannel;
}

export interface RoomPublicChannel extends RoomBase, RoomMUC, RoomChannel {
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
    room: SdkRoom<DirectMessage>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGroup {
    kind: RoomType,
    room: SdkRoom<Group>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPrivateChannel {
    kind: RoomType,
    room: SdkRoom<PrivateChannel>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomPublicChannel {
    kind: RoomType,
    room: SdkRoom<PublicChannel>,
}

#[wasm_bindgen(skip_typescript)]
pub struct RoomGeneric {
    kind: RoomType,
    room: SdkRoom<Generic>,
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

            #[wasm_bindgen(getter)]
            pub fn occupants(&self) -> BareJidArray {
                self.room
                    .occupants()
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
            pub async fn load_latest_messages(&self) -> Result<MessagesArray> {
                let messages = self
                    .room
                    .load_latest_messages()
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
                let message_id_refs = message_ids.iter().collect::<Vec<_>>();

                let messages = self
                    .room
                    .load_messages_with_ids(&message_id_refs)
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

            #[wasm_bindgen(js_name = "loadComposingUsers")]
            pub async fn load_composing_users(&self) -> Result<BareJidArray> {
                Ok(self
                    .room
                    .load_composing_users()
                    .await
                    .map_err(WasmError::from)?
                    .into_iter()
                    .map(BareJid::from)
                    .collect_into_js_array::<BareJidArray>())
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

            #[wasm_bindgen(js_name = "setTopic")]
            pub async fn set_topic(&self, topic: Option<String>) -> Result<()> {
                self.room
                    .set_topic(topic.as_deref())
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }
        }
    };
}

macro_rules! channel_room_impl {
    ($t:ident) => {
        #[wasm_bindgen]
        impl $t {
            #[wasm_bindgen(js_name = "inviteUsers")]
            pub async fn invite_users(&self, users: Array) -> Result<()> {
                let users = try_jid_vec_from_string_array(users)?;
                self.room
                    .invite_users(users.as_slice())
                    .await
                    .map_err(WasmError::from)?;
                Ok(())
            }
        }
    };
}

#[wasm_bindgen]
impl RoomGroup {
    #[wasm_bindgen(js_name = "resendInvitesToMembers")]
    pub async fn resend_invites_to_members(&self) -> Result<()> {
        self.room
            .resend_invites_to_members()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }
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

channel_room_impl!(RoomPrivateChannel);
channel_room_impl!(RoomPublicChannel);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Room[]")]
    pub type RoomsArray;
}

pub trait RoomEnvelopeExt {
    fn into_js_value(self) -> JsValue;
}

impl RoomEnvelopeExt for RoomEnvelope {
    fn into_js_value(self) -> JsValue {
        match self {
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
    }
}

impl From<Vec<RoomEnvelope>> for RoomsArray {
    fn from(value: Vec<RoomEnvelope>) -> Self {
        value
            .into_iter()
            .map(|envelope| envelope.into_js_value())
            .collect_into_js_array::<RoomsArray>()
    }
}
