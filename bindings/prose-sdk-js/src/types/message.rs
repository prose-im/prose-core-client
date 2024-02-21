// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::*;

use prose_core_client::dtos;
use prose_core_client::dtos::StanzaId;

use crate::types::{Attachment, AttachmentsArray, BareJid, IntoJSArray};

use super::{BareJidArray, ReactionsArray};

#[wasm_bindgen]
pub struct ArchiveID(StanzaId);

impl AsRef<StanzaId> for ArchiveID {
    fn as_ref(&self) -> &StanzaId {
        &self.0
    }
}

#[wasm_bindgen]
pub struct Message(dtos::Message);

#[wasm_bindgen]
pub struct Reaction {
    #[wasm_bindgen(skip)]
    pub emoji: String,
    #[wasm_bindgen(skip)]
    pub from: Vec<String>,
}

#[wasm_bindgen]
pub struct MessageMetadata {
    #[wasm_bindgen(js_name = "isEdited")]
    pub is_edited: bool,
}

#[wasm_bindgen]
pub struct MessageSender(dtos::MessageSender);

#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Option<String> {
        self.0.id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: Option<String>) {
        self.0.id = id.clone().map(Into::into)
    }

    #[wasm_bindgen(getter, js_name = "archiveId")]
    pub fn stanza_id(&self) -> Option<ArchiveID> {
        self.0.stanza_id.as_ref().map(|id| ArchiveID(id.clone()))
    }

    #[wasm_bindgen(getter, js_name = "from")]
    pub fn from_(&self) -> String {
        self.0.from.id.to_opaque_identifier()
    }

    #[wasm_bindgen(getter, js_name = "user")]
    pub fn user(&self) -> MessageSender {
        MessageSender(self.0.from.clone())
    }

    #[wasm_bindgen(getter, js_name = "content")]
    pub fn body(&self) -> String {
        if self.0.body.is_empty() {
            return "<empty message>".to_string();
        }
        self.0.body.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn date(&self) -> js_sys::Date {
        let timestamp_ms = self.0.timestamp.timestamp_millis() as f64;
        let js_date = js_sys::Date::new(&JsValue::from(timestamp_ms));
        js_date
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
        MessageMetadata {
            is_edited: self.0.is_edited,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn reactions(&self) -> ReactionsArray {
        self.0
            .reactions
            .iter()
            .map(|r| Reaction::from(r.clone()))
            .collect_into_js_array::<ReactionsArray>()
    }

    #[wasm_bindgen(getter)]
    pub fn attachments(&self) -> AttachmentsArray {
        self.0
            .attachments
            .iter()
            .map(|a| Attachment::from(a.clone()))
            .collect_into_js_array::<AttachmentsArray>()
    }
}

#[wasm_bindgen]
impl Reaction {
    #[wasm_bindgen(getter, js_name = "reaction")]
    pub fn emoji(&self) -> String {
        self.emoji.clone()
    }

    #[wasm_bindgen(getter, js_name = "authors")]
    pub fn from_(&self) -> BareJidArray {
        self.from
            .iter()
            .cloned()
            .collect_into_js_array::<BareJidArray>()
    }
}

#[wasm_bindgen]
impl MessageSender {
    /// An opaque ID to identify the message sender. Should be used to group messages of
    /// the same sender.
    #[wasm_bindgen(getter, js_name = "jid")]
    pub fn sender_id(&self) -> String {
        self.0.id.to_opaque_identifier()
    }

    /// The real ID of the message sender, if available.
    #[wasm_bindgen(getter, js_name = "userID")]
    pub fn user_id(&self) -> Option<BareJid> {
        match &self.0.id {
            dtos::ParticipantId::User(id) => Some(id.clone().into_inner().into()),
            dtos::ParticipantId::Occupant(_) => None,
        }
    }

    /// The name of the message sender.
    #[wasm_bindgen(getter, js_name = "name")]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }
}

impl From<dtos::Message> for Message {
    fn from(value: dtos::Message) -> Self {
        Message(value)
    }
}

impl From<dtos::Reaction> for Reaction {
    fn from(value: dtos::Reaction) -> Self {
        Reaction {
            emoji: value.emoji.into_inner(),
            from: value
                .from
                .into_iter()
                .map(|id| id.to_opaque_identifier())
                .collect(),
        }
    }
}
