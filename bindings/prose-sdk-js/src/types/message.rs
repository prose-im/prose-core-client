// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::error;
use wasm_bindgen::prelude::*;

use prose_core_client::dtos;
use prose_core_client::dtos::{ScalarRangeExt, StanzaId};

use crate::types::{
    Attachment, AttachmentsArray, BareJid, IntoJSArray, Mention, MentionsArray, MessageSendersArray,
};

use super::ReactionsArray;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ArchiveID(StanzaId);

impl From<StanzaId> for ArchiveID {
    fn from(value: StanzaId) -> Self {
        Self(value)
    }
}

impl AsRef<StanzaId> for ArchiveID {
    fn as_ref(&self) -> &StanzaId {
        &self.0
    }
}

#[wasm_bindgen]
pub struct Message {
    id: Option<String>,
    stanza_id: Option<ArchiveID>,
    from: MessageSender,
    body: String,
    timestamp: js_sys::Date,
    meta: MessageMetadata,
    reactions: js_sys::Array,
    attachments: js_sys::Array,
    mentions: js_sys::Array,
}

#[wasm_bindgen]
pub struct Reaction {
    #[wasm_bindgen(skip)]
    pub emoji: String,
    #[wasm_bindgen(skip)]
    pub from: Vec<MessageSender>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MessageMetadata {
    #[wasm_bindgen(js_name = "isEdited")]
    pub is_edited: bool,
    #[wasm_bindgen(js_name = "isTransient")]
    pub is_transient: bool,
    #[wasm_bindgen(js_name = "isEncrypted")]
    pub is_encrypted: bool,
    #[wasm_bindgen(js_name = "isLastRead")]
    /// When contained in a list, this message is the last message that our user has read.
    pub is_last_read: bool,
}

impl From<dtos::Message> for Message {
    fn from(value: dtos::Message) -> Self {
        let mentions = value
            .mentions
            .into_iter()
            .filter_map(|mention| {
                let Ok(range) = mention.range.to_utf16_range(&value.body) else {
                    error!("Failed to convert mention range");
                    return None;
                };
                Some(Mention {
                    user: mention.user.into_inner().into(),
                    range,
                })
            })
            .collect_into_js_array();

        Self {
            id: value.id.map(|id| id.to_string()),
            stanza_id: value.stanza_id.map(ArchiveID::from),
            from: value.from.into(),
            body: value.body,
            timestamp: js_sys::Date::new(&JsValue::from(value.timestamp.timestamp_millis() as f64)),
            meta: MessageMetadata {
                is_edited: value.is_edited,
                is_transient: value.is_transient,
                is_encrypted: value.is_encrypted,
                is_last_read: value.is_last_read,
            },
            reactions: value
                .reactions
                .into_iter()
                .map(Reaction::from)
                .collect_into_js_array(),
            attachments: value
                .attachments
                .into_iter()
                .map(Attachment::from)
                .collect_into_js_array(),
            mentions,
        }
    }
}

#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Option<String> {
        self.id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: Option<String>) {
        self.id = id.clone().map(Into::into)
    }

    #[wasm_bindgen(getter, js_name = "archiveId")]
    pub fn stanza_id(&self) -> Option<ArchiveID> {
        self.stanza_id.clone()
    }

    #[wasm_bindgen(getter, js_name = "from")]
    pub fn from_(&self) -> String {
        self.from.id.to_opaque_identifier()
    }

    #[wasm_bindgen(getter, js_name = "user")]
    pub fn user(&self) -> MessageSender {
        self.from.clone()
    }

    #[wasm_bindgen(getter, js_name = "content")]
    pub fn body(&self) -> String {
        if self.body.is_empty() {
            return "<empty message>".to_string();
        }
        self.body.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn date(&self) -> js_sys::Date {
        self.timestamp.clone()
    }
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn _type(&self) -> String {
        "text".to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.body()
    }
    #[wasm_bindgen(getter)]
    pub fn meta(&self) -> MessageMetadata {
        self.meta.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn reactions(&self) -> ReactionsArray {
        self.reactions.clone().unchecked_into()
    }

    #[wasm_bindgen(getter)]
    pub fn attachments(&self) -> AttachmentsArray {
        self.attachments.clone().unchecked_into()
    }

    #[wasm_bindgen(getter)]
    pub fn mentions(&self) -> MentionsArray {
        self.mentions.clone().unchecked_into()
    }
}

#[wasm_bindgen]
impl Reaction {
    #[wasm_bindgen(getter, js_name = "reaction")]
    pub fn emoji(&self) -> String {
        self.emoji.clone()
    }

    #[wasm_bindgen(getter, js_name = "authors")]
    pub fn from_(&self) -> MessageSendersArray {
        self.from
            .iter()
            .cloned()
            .collect_into_js_array::<MessageSendersArray>()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct MessageSender {
    id: dtos::ParticipantId,
    name: String,
}

impl From<dtos::MessageSender> for MessageSender {
    fn from(value: dtos::MessageSender) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[wasm_bindgen]
impl MessageSender {
    /// An opaque ID to identify the message sender. Should be used to group messages of
    /// the same sender.
    #[wasm_bindgen(getter, js_name = "jid")]
    pub fn sender_id(&self) -> String {
        self.id.to_opaque_identifier()
    }

    /// The real ID of the message sender, if available.
    #[wasm_bindgen(getter, js_name = "userID")]
    pub fn user_id(&self) -> Option<BareJid> {
        match &self.id {
            dtos::ParticipantId::User(id) => Some(id.clone().into_inner().into()),
            dtos::ParticipantId::Occupant(_) => None,
        }
    }

    /// The name of the message sender.
    #[wasm_bindgen(getter, js_name = "name")]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl From<dtos::Reaction> for Reaction {
    fn from(value: dtos::Reaction) -> Self {
        Reaction {
            emoji: value.emoji.into_inner(),
            from: value.from.into_iter().map(MessageSender::from).collect(),
        }
    }
}
