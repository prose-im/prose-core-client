// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::bookmarks2::Conference;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub;
use xmpp_parsers::pubsub::pubsub::Notify;
use xmpp_parsers::pubsub::{NodeName, PubSubEvent};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::ns;
use crate::stanza::ConferenceBookmark;
use crate::util::{PublishOptionsExt, RequestError};
use crate::Event as ClientEvent;

/// XEP-0402: PEP Native Bookmarks
/// https://xmpp.org/extensions/xep-0402.html
#[derive(Default, Clone)]
pub struct Bookmark {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    BookmarksPublished { bookmarks: Vec<ConferenceBookmark> },
    BookmarksRetracted { jids: Vec<Jid> },
}

impl Module for Bookmark {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_pubsub_event(&self, _from: &Jid, event: &PubSubEvent) -> Result<()> {
        match event {
            PubSubEvent::PublishedItems { node, items } => {
                if node.0 == ns::BOOKMARKS2 {
                    return self.handle_published_bookmarks(items);
                }
            }
            PubSubEvent::RetractedItems { node, items } => {
                if node.0 == ns::BOOKMARKS2 {
                    return self.handle_retracted_bookmarks(items);
                }
            }
            _ => (),
        }
        Ok(())
    }
}

impl Bookmark {
    pub async fn load_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            pubsub::PubSub::Items(pubsub::pubsub::Items {
                max_items: None,
                node: NodeName(ns::BOOKMARKS2.to_string()),
                subid: None,
                items: vec![],
            }),
        );

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        let pubsub::PubSub::Items(items) = pubsub::PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        let bookmarks = items
            .items
            .into_iter()
            .map(|item| ConferenceBookmark::try_from(item.0))
            .collect::<Result<Vec<ConferenceBookmark>>>()?;

        Ok(bookmarks)
    }

    // Use this method to either save or update a bookmark.
    // Updating a bookmark means republishing it with the same bookmark JID.
    pub async fn publish_bookmark(&self, jid: Jid, conference: Conference) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::BOOKMARKS2.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: Some(pubsub::ItemId(jid.to_string())),
                        publisher: None,
                        payload: Some(conference.into()),
                    })],
                },
                publish_options: Some(pubsub::pubsub::PublishOptions::for_private_data()),
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn retract_bookmark(&self, jid: Jid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Retract(pubsub::pubsub::Retract {
                node: NodeName(ns::BOOKMARKS2.to_string()),
                notify: Notify::True,
                items: vec![pubsub::pubsub::Item(pubsub::Item {
                    id: Some(pubsub::ItemId(jid.to_string())),
                    publisher: None,
                    payload: None,
                })],
            }),
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }
}

impl Bookmark {
    fn handle_published_bookmarks(&self, items: &Vec<pubsub::event::Item>) -> Result<()> {
        let bookmarks = items
            .iter()
            .map(|item| ConferenceBookmark::try_from(item.0.clone()))
            .collect::<Result<Vec<ConferenceBookmark>>>()?;

        self.ctx
            .schedule_event(ClientEvent::Bookmark2(Event::BookmarksPublished {
                bookmarks,
            }));

        Ok(())
    }

    fn handle_retracted_bookmarks(&self, items: &Vec<pubsub::ItemId>) -> Result<()> {
        let jids = items
            .iter()
            .map(|id| Jid::from_str(&id.0))
            .collect::<Result<Vec<Jid>, _>>()?;
        self.ctx
            .schedule_event(ClientEvent::Bookmark2(Event::BookmarksRetracted { jids }));
        Ok(())
    }
}
