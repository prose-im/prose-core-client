// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use itertools::partition;
use tracing::{info, warn};
use xmpp_parsers::pubsub::event::Item;
use xmpp_parsers::pubsub::{ItemId, PubSubEvent};

use prose_xmpp::mods::pubsub::Event as XMPPPubSubEvent;

use crate::app::event_handlers::SidebarBookmarkEvent;
use crate::domain::shared::models::RoomId;
use crate::domain::sidebar::models::Bookmark;
use crate::infra::xmpp::event_parser::Context;
use crate::infra::xmpp::type_conversions::bookmark::ns;

pub fn parse_pubsub_event(ctx: &mut Context, event: XMPPPubSubEvent) -> Result<()> {
    match event {
        XMPPPubSubEvent::PubSubMessage { mut message } => {
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
            parse_pubsub_events(ctx, message.events.drain(partition_idx..))?;

            if !message.events.is_empty() {
                info!("PubSub message contains unhandled events.")
            }

            Ok(())
        }
    }
}

fn parse_pubsub_events(
    ctx: &mut Context,
    events: impl IntoIterator<Item = PubSubEvent>,
) -> Result<()> {
    for event in events {
        match event {
            PubSubEvent::PublishedItems { items, .. } => handle_added_or_updated_items(ctx, items)?,
            PubSubEvent::RetractedItems { items, .. } => handle_retracted_items(ctx, items)?,
            PubSubEvent::Purge { .. } | PubSubEvent::Delete { .. } => {
                ctx.push_event(SidebarBookmarkEvent::Purged)
            }
            PubSubEvent::Configuration { .. } => {}
            PubSubEvent::Subscription { .. } => {}
        }
    }

    Ok(())
}

fn handle_added_or_updated_items(ctx: &mut Context, items: Vec<Item>) -> Result<()> {
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

    ctx.push_event(SidebarBookmarkEvent::AddedOrUpdated { bookmarks });
    Ok(())
}

fn handle_retracted_items(ctx: &mut Context, ids: Vec<ItemId>) -> Result<()> {
    let ids = ids
        .into_iter()
        .map(|id| id.0.parse::<RoomId>())
        .collect::<Result<Vec<_>, _>>()?;

    ctx.push_event(SidebarBookmarkEvent::Deleted { ids });
    Ok(())
}
