// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Result;
use jid::BareJid;
use tracing::{error, warn};
use xmpp_parsers::nick::Nick;
use xmpp_parsers::pubsub;

use crate::app::event_handlers::{PubSubEventType, UserInfoEvent, UserInfoEventType};
use crate::domain::encryption::models::DeviceList;
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::sidebar::models::Bookmark;
use crate::dtos::{RoomId, UserId};
use crate::infra::xmpp::event_parser::pubsub::generic_pub_sub_parser::GenericPubSubParser;
use crate::infra::xmpp::event_parser::Context;
use crate::infra::xmpp::type_conversions::{bookmark, synced_room_settings};
use prose_xmpp::mods::pubsub::Event as XMPPPubSubEvent;
use prose_xmpp::ns;

mod generic_pub_sub_parser;

pub(self) trait PubSubParser: Send + Sync {
    fn handle_added_or_updated_items(
        &self,
        ctx: &mut Context,
        from: &UserId,
        items: Vec<pubsub::event::Item>,
    ) -> Result<()>;

    fn handle_retracted_items(
        &self,
        ctx: &mut Context,
        from: &UserId,
        items: Vec<pubsub::ItemId>,
    ) -> Result<()>;

    fn handle_purge(&self, ctx: &mut Context, from: &UserId) -> Result<()>;
}

static PUB_SUB_PARSERS: OnceLock<HashMap<String, Box<dyn PubSubParser>>> = OnceLock::new();

fn get_parser(ns: &str) -> Option<&Box<dyn PubSubParser>> {
    PUB_SUB_PARSERS
        .get_or_init(|| {
            [
                (
                    bookmark::ns::PROSE_BOOKMARK.to_string(),
                    Box::new(GenericPubSubParser::<BareJid, Bookmark>::new(Into::into))
                        as Box<dyn PubSubParser>,
                ),
                (
                    ns::LEGACY_OMEMO_DEVICELIST.to_string(),
                    Box::new(GenericPubSubParser::<String, DeviceList>::new(Into::into)),
                ),
                (
                    synced_room_settings::ns::PROSE_ROOM_SETTINGS.to_string(),
                    Box::new(GenericPubSubParser::<RoomId, SyncedRoomSettings>::new(
                        Into::into,
                    )) as Box<dyn PubSubParser>,
                ),
                (
                    ns::NICK.to_string(),
                    Box::new(GenericPubSubParser::<String, Nick>::new(|item| {
                        let nickname = match item.r#type {
                            PubSubEventType::AddedOrUpdated { mut items } => {
                                items.pop().map(|nick| nick.0)
                            }
                            PubSubEventType::Deleted { .. } => None,
                            PubSubEventType::Purged => None,
                        };

                        UserInfoEvent {
                            user_id: item.user_id,
                            r#type: UserInfoEventType::NicknameChanged { nickname },
                        }
                        .into()
                    })) as Box<dyn PubSubParser>,
                ),
            ]
            .into_iter()
            .collect()
        })
        .get(ns)
}

pub fn parse_pubsub_event(ctx: &mut Context, event: XMPPPubSubEvent) -> Result<()> {
    match event {
        XMPPPubSubEvent::PubSubMessage { message } => {
            let from = UserId::from(message.from.into_bare());

            let grouped_events =
                message
                    .events
                    .into_iter()
                    .fold(HashMap::new(), |mut events, event| {
                        let node = match &event {
                            pubsub::PubSubEvent::Configuration { node, .. } => node,
                            pubsub::PubSubEvent::Delete { node, .. } => node,
                            pubsub::PubSubEvent::PublishedItems { node, .. } => node,
                            pubsub::PubSubEvent::RetractedItems { node, .. } => node,
                            pubsub::PubSubEvent::Purge { node, .. } => node,
                            pubsub::PubSubEvent::Subscription { node, .. } => node,
                        };
                        events
                            .entry(node.0.clone())
                            .or_insert_with(Vec::new)
                            .push(event);
                        events
                    });

            for (ns, events) in grouped_events {
                match parse_pubsub_events(ctx, &ns, &from, events) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Failed to parse '{ns}' PubSub events: {err}")
                    }
                }
            }

            Ok(())
        }
    }
}

fn parse_pubsub_events(
    ctx: &mut Context,
    ns: &str,
    from: &UserId,
    events: impl IntoIterator<Item = pubsub::PubSubEvent>,
) -> Result<()> {
    let Some(parser) = get_parser(&ns) else {
        warn!("No PubSub parser for node '{ns}'.");
        return Ok(());
    };

    for event in events {
        match event {
            pubsub::PubSubEvent::PublishedItems { items, .. } => {
                parser.handle_added_or_updated_items(ctx, from, items)?
            }
            pubsub::PubSubEvent::RetractedItems { items, .. } => {
                parser.handle_retracted_items(ctx, from, items)?
            }
            pubsub::PubSubEvent::Purge { .. } | pubsub::PubSubEvent::Delete { .. } => {
                parser.handle_purge(ctx, from)?
            }
            pubsub::PubSubEvent::Configuration { .. } => {}
            pubsub::PubSubEvent::Subscription { .. } => {}
        }
    }

    Ok(())
}
