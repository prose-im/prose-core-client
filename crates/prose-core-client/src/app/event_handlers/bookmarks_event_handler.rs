// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use itertools::partition;
use tracing::warn;
use xmpp_parsers::pubsub::event::Item;
use xmpp_parsers::pubsub::{ItemId, PubSubEvent};

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::pubsub;
use prose_xmpp::Event;

use crate::app::deps::DynSidebarDomainService;
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::sidebar::models::Bookmark;
use crate::dtos::RoomId;
use crate::infra::xmpp::type_conversions::bookmark::ns;

#[derive(InjectDependencies)]
pub struct BookmarksEventHandler {
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl XMPPEventHandler for BookmarksEventHandler {
    fn name(&self) -> &'static str {
        "bookmarks"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::PubSub(event) => match event {
                pubsub::Event::PubSubMessage { mut message } => {
                    let partition_idx = partition(&mut message.events, |event| {
                        let node = match event {
                            PubSubEvent::Configuration { node, .. } => node,
                            PubSubEvent::Delete { node, .. } => node,
                            PubSubEvent::PublishedItems { node, .. } => node,
                            PubSubEvent::RetractedItems { node, .. } => node,
                            PubSubEvent::Purge { node, .. } => node,
                            PubSubEvent::Subscription { node, .. } => node,
                        };
                        node.0 != ns::PROSE_BOOKMARK
                    });
                    self.handle_pubsub_events(message.events.drain(partition_idx..))
                        .await?;

                    if message.events.is_empty() {
                        return Ok(None);
                    }

                    Ok(Some(Event::PubSub(pubsub::Event::PubSubMessage { message })))
                }
            },
            _ => Ok(Some(event)),
        }
    }
}

impl BookmarksEventHandler {
    async fn handle_pubsub_events(
        &self,
        events: impl IntoIterator<Item = PubSubEvent>,
    ) -> Result<()> {
        for event in events {
            match event {
                PubSubEvent::PublishedItems { items, .. } => {
                    self.handle_added_or_updated_items(items).await?
                }
                PubSubEvent::RetractedItems { items, .. } => {
                    self.handle_retracted_items(items).await?
                }
                PubSubEvent::Purge { .. } | PubSubEvent::Delete { .. } => {
                    self.handle_purge().await?
                }
                PubSubEvent::Configuration { .. } => {}
                PubSubEvent::Subscription { .. } => {}
            }
        }

        Ok(())
    }

    async fn handle_added_or_updated_items(&self, items: Vec<Item>) -> Result<()> {
        let bookmarks = items
            .into_iter()
            .filter_map(|item| {
                let Some(payload) = item.0.payload else {
                    warn!("Encountered missing payload in PubSub item for bookmark");
                    return None;
                };

                let Ok(bookmark) = Bookmark::try_from(payload) else {
                    warn!("Encountered invalid payload in PubSub item for bookmark");
                    return None;
                };

                Some(bookmark)
            })
            .collect::<Vec<_>>();

        self.sidebar_domain_service
            .extend_items_from_bookmarks(bookmarks)
            .await?;
        Ok(())
    }

    async fn handle_retracted_items(&self, ids: Vec<ItemId>) -> Result<()> {
        let jids = ids
            .into_iter()
            .map(|id| RoomId::from_str(&id.0))
            .collect::<Result<Vec<_>, _>>()?;
        let jids_refs = jids.iter().collect::<Vec<_>>();

        self.sidebar_domain_service
            .handle_removed_items(jids_refs.as_slice())
            .await?;

        Ok(())
    }

    async fn handle_purge(&self) -> Result<()> {
        self.sidebar_domain_service.handle_remote_purge().await?;
        Ok(())
    }
}
