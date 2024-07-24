// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use xmpp_parsers::mam::Complete;

use prose_xmpp::mods;
use prose_xmpp::stanza::mam::query;
use prose_xmpp::stanza::mam::query::{DateTimeFilter, RangeFilter};
use prose_xmpp::stanza::message::stanza_id;

use crate::domain::messaging::models::MessageServerId;
use crate::domain::messaging::services::{MessageArchiveService, MessagePage};
use crate::dtos::RoomId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageArchiveService for XMPPClient {
    async fn load_messages_before(
        &self,
        room_id: &RoomId,
        before: Option<&MessageServerId>,
        batch_size: u32,
    ) -> Result<MessagePage> {
        let mam = self.client.get_mod::<mods::MAM>();
        let before: Option<stanza_id::Id> = before.map(|id| id.as_ref().into());

        let rsm_filter = query::RsmFilter {
            range: Some(query::RsmRange::Before(before)),
            max: Some(batch_size as usize),
        };

        let mut query = query::Query {
            filter: None,
            rsm_filter: Some(rsm_filter),
            flip_page: false,
        };

        let to = match room_id {
            RoomId::User(id) => {
                query.filter = Some(query::Filter {
                    range: None,
                    with: Some(id.as_ref().clone().into()),
                });
                None
            }
            RoomId::Muc(id) => Some(id.as_ref()),
        };

        let (messages, fin) = mam.load_messages(to, query).await?;

        Ok(MessagePage {
            messages,
            is_last: fin.complete == Complete::True,
        })
    }

    async fn load_messages_after(
        &self,
        room_id: &RoomId,
        after: &MessageServerId,
        batch_size: u32,
    ) -> Result<MessagePage> {
        let mam = self.client.get_mod::<mods::MAM>();
        let after: stanza_id::Id = after.as_ref().into();

        let rsm_filter = query::RsmFilter {
            range: Some(query::RsmRange::After(after)),
            max: Some(batch_size as usize),
        };

        let mut query = query::Query {
            filter: None,
            rsm_filter: Some(rsm_filter),
            flip_page: false,
        };

        let to = match room_id {
            RoomId::User(id) => {
                query.filter = Some(query::Filter {
                    range: None,
                    with: Some(id.as_ref().clone().into()),
                });
                None
            }
            RoomId::Muc(id) => Some(id.as_ref()),
        };

        let (messages, fin) = mam.load_messages(to, query).await?;

        Ok(MessagePage {
            messages,
            is_last: fin.complete == Complete::True,
        })
    }

    async fn load_messages_since(
        &self,
        room_id: &RoomId,
        since: DateTime<Utc>,
        batch_size: u32,
    ) -> Result<MessagePage> {
        let mam = self.client.get_mod::<mods::MAM>();

        let mut query = query::Query {
            filter: None,
            rsm_filter: Some(query::RsmFilter {
                range: None,
                max: Some(batch_size as usize),
            }),
            flip_page: false,
        };

        let to = match room_id {
            RoomId::User(id) => {
                query.filter = Some(query::Filter {
                    range: Some(RangeFilter::DateTime(DateTimeFilter::Start(since))),
                    with: Some(id.as_ref().clone().into()),
                });
                None
            }
            RoomId::Muc(id) => {
                query.filter = Some(query::Filter {
                    range: Some(RangeFilter::DateTime(DateTimeFilter::Start(since))),
                    with: None,
                });
                Some(id.as_ref())
            }
        };

        let (messages, fin) = mam.load_messages(to, query).await?;

        Ok(MessagePage {
            messages,
            is_last: fin.complete == Complete::True,
        })
    }
}
