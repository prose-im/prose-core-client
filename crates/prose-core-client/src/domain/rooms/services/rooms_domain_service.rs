// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::{RoomError, RoomInternals};
use crate::domain::shared::models::RoomJid;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateRoomType {
    Group {
        participants: Vec<BareJid>,
        send_invites: bool,
    },
    PrivateChannel {
        name: String,
    },
    PublicChannel {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateOrEnterRoomRequestType {
    Create {
        service: BareJid,
        room_type: CreateRoomType,
    },
    Join {
        room_jid: RoomJid,
        nickname: Option<String>,
        password: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateOrEnterRoomRequest {
    pub r#type: CreateOrEnterRoomRequestType,
    pub save_bookmark: bool,
    pub insert_sidebar_item: bool,
    pub notify_delegate: bool,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomsDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn create_or_join_room(
        &self,
        request: CreateOrEnterRoomRequest,
    ) -> Result<Arc<RoomInternals>, RoomError>;
}
