// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomTopicService {
    async fn set_topic(&self, room_jid: &BareJid, subject: Option<&str>) -> Result<()>;
}
