// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use minidom::Element;
use tracing::warn;
use xmpp_parsers::pubsub;

use crate::app::event_handlers::{PubSubEvent, PubSubEventType, ServerEvent};
use crate::dtos::UserId;
use crate::infra::xmpp::event_parser::Context;

use super::PubSubParser;

pub struct GenericPubSubParser<Id, Item> {
    id_phantom: PhantomData<Id>,
    item_phantom: PhantomData<Item>,
    to_server_event: fn(PubSubEvent<Id, Item>) -> ServerEvent,
}

impl<Id, Item> GenericPubSubParser<Id, Item> {
    pub fn new(to_server_event: fn(PubSubEvent<Id, Item>) -> ServerEvent) -> Self {
        Self {
            id_phantom: Default::default(),
            item_phantom: Default::default(),
            to_server_event,
        }
    }
}

impl<Id, Item> PubSubParser for GenericPubSubParser<Id, Item>
where
    Id: FromStr + Clone + PartialEq + Send + Sync,
    <Id as FromStr>::Err: Send + Sync + Display,
    Item: TryFrom<Element> + Clone + PartialEq + Send + Sync,
{
    fn handle_added_or_updated_items(
        &self,
        ctx: &mut Context,
        from: &UserId,
        items: Vec<pubsub::event::Item>,
    ) -> Result<()> {
        let items = items
            .into_iter()
            .filter_map(|item| {
                let Some(payload) = item.payload else {
                    warn!("Encountered missing payload in PubSub item for bookmark");
                    return None;
                };

                let Ok(bookmark) = Item::try_from(payload) else {
                    warn!("Encountered invalid payload in PubSub item for bookmark");
                    return None;
                };

                Some(bookmark)
            })
            .collect::<Vec<_>>();

        ctx.push_event((self.to_server_event)(PubSubEvent {
            user_id: from.clone(),
            r#type: PubSubEventType::AddedOrUpdated { items },
        }));
        Ok(())
    }

    fn handle_retracted_items(
        &self,
        ctx: &mut Context,
        from: &UserId,
        ids: Vec<pubsub::ItemId>,
    ) -> Result<()> {
        let ids = ids
            .into_iter()
            .map(|id| match id.0.parse::<Id>() {
                Ok(id) => Ok(id),
                Err(err) => Err(anyhow!("Failed to parse pubsub id '{}': {err}", id.0)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        ctx.push_event((self.to_server_event)(PubSubEvent {
            user_id: from.clone(),
            r#type: PubSubEventType::Deleted { ids },
        }));
        Ok(())
    }

    fn handle_purge(&self, ctx: &mut Context, from: &UserId) -> Result<()> {
        ctx.push_event((self.to_server_event)(PubSubEvent {
            user_id: from.clone(),
            r#type: PubSubEventType::Purged,
        }));
        Ok(())
    }
}
