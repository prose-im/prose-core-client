// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

use prose_core_client::dtos::SidebarItem as SidebarItemDTO;
use prose_core_client::services::RoomEnvelope;
use prose_core_client::Client;

use crate::client::WasmError;
use crate::types::RoomEnvelopeExt;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum SidebarSection {
    Favorites = 0,
    DirectMessage = 1,
    Channel = 2,
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface SidebarItem {
    readonly section: SidebarSection;
    readonly name: string;
    readonly room: Room;
    readonly isFavorite: boolean;
    readonly hasDraft: boolean;
    readonly unreadCount: number;
    readonly error?: string;
    
    toggleFavorite(): Promise<void>;
    removeFromSidebar(): Promise<void>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SidebarItem[]")]
    pub type SidebarItemsArray;
}

#[wasm_bindgen(skip_typescript)]
pub struct SidebarItem {
    #[wasm_bindgen(skip)]
    pub dto: SidebarItemDTO,
    #[wasm_bindgen(skip)]
    pub client: Client,
}

#[wasm_bindgen]
impl SidebarItem {
    #[wasm_bindgen(getter)]
    pub fn section(&self) -> SidebarSection {
        match &self.dto.room {
            _ if self.dto.is_favorite => SidebarSection::Favorites,
            RoomEnvelope::DirectMessage(_) => SidebarSection::DirectMessage,
            RoomEnvelope::Group(_) => SidebarSection::DirectMessage,
            RoomEnvelope::PrivateChannel(_) => SidebarSection::Channel,
            RoomEnvelope::PublicChannel(_) => SidebarSection::Channel,
            RoomEnvelope::Generic(_) => unreachable!("Unexpected Sidebar item for generic room"),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.dto.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn room(&self) -> JsValue {
        self.dto.room.clone().into_js_value()
    }

    #[wasm_bindgen(getter, js_name = "isFavorite")]
    pub fn is_favorite(&self) -> bool {
        self.dto.is_favorite
    }

    #[wasm_bindgen(getter, js_name = "hasDraft")]
    pub fn has_draft(&self) -> bool {
        self.dto.has_draft
    }

    #[wasm_bindgen(getter, js_name = "unreadCount")]
    pub fn unread_count(&self) -> u32 {
        self.dto.unread_count
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.dto.error.clone()
    }
}

#[wasm_bindgen]
impl SidebarItem {
    #[wasm_bindgen(js_name = "toggleFavorite")]
    pub async fn toggle_favorite(&self) -> Result<(), JsError> {
        self.client
            .sidebar
            .toggle_favorite(self.dto.room.to_generic_room().jid())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeFromSidebar")]
    pub async fn remove_from_sidebar(&self) -> Result<(), JsError> {
        self.client
            .sidebar
            .remove_from_sidebar(self.dto.room.to_generic_room().jid())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }
}
