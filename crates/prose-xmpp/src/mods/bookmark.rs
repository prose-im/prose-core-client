// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::ns;
use crate::util::{PublishOptionsExt, RequestError};
use crate::Event as ClientEvent;
use anyhow::{bail, Result};
use jid::Jid;
use std::str::FromStr;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub::pubsub::Notify;
use xmpp_parsers::pubsub::{NodeName, PubSubEvent};
use xmpp_parsers::{bookmarks, pubsub};

/// XEP-0402: PEP Native Bookmarks
/// https://xmpp.org/extensions/xep-0402.html
#[derive(Default, Clone)]
pub struct Bookmark {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    BookmarksPublished {
        bookmarks: Vec<ConferenceBookmark>,
    },
    BookmarksRetracted {
        jids: Vec<Jid>,
    },
    /// This event will only be sent when a notification according to XEP-0048 is received.
    BookmarksReplaced {
        bookmarks: Vec<ConferenceBookmark>,
    },
}

#[derive(Debug, Clone)]
pub struct ConferenceBookmark {
    pub jid: Jid,
    pub conference: Conference,
}

impl TryFrom<pubsub::Item> for ConferenceBookmark {
    type Error = anyhow::Error;

    fn try_from(item: pubsub::Item) -> Result<Self> {
        let Some(id) = &item.id else {
            bail!("Missing id in bookmark");
        };
        let Some(payload) = &item.payload else {
            bail!("Missing payload in bookmark");
        };

        let jid = Jid::from_str(&id.0).map_err(anyhow::Error::new)?;
        let conference = Conference::try_from(payload.clone()).map_err(anyhow::Error::new)?;

        Ok(ConferenceBookmark { jid, conference })
    }
}

impl From<bookmarks::Conference> for ConferenceBookmark {
    fn from(conference: bookmarks::Conference) -> Self {
        ConferenceBookmark {
            jid: Jid::Bare(conference.jid),
            conference: Conference {
                autojoin: if conference.autojoin == bookmarks::Autojoin::True {
                    Autojoin::True
                } else {
                    Autojoin::False
                },
                name: conference.name,
                nick: conference.nick,
                password: conference.password,
                extensions: vec![],
            },
        }
    }
}

impl From<ConferenceBookmark> for bookmarks::Conference {
    fn from(bookmark: ConferenceBookmark) -> Self {
        bookmarks::Conference {
            autojoin: if bookmark.conference.autojoin == Autojoin::True {
                bookmarks::Autojoin::True
            } else {
                bookmarks::Autojoin::False
            },
            jid: bookmark.jid.into_bare(),
            name: bookmark.conference.name,
            nick: bookmark.conference.nick,
            password: bookmark.conference.password,
        }
    }
}

impl PartialEq for ConferenceBookmark {
    fn eq(&self, other: &Self) -> bool {
        self.jid == other.jid
            && self.conference.autojoin == other.conference.autojoin
            && self.conference.name == other.conference.name
            && self.conference.nick == other.conference.nick
            && self.conference.password == other.conference.password
            && self.conference.extensions == other.conference.extensions
    }
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
                } else if node.0 == ns::BOOKMARKS {
                    return self.handle_legacy_bookmarks(items);
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

        let response = self
            .ctx
            .send_iq(iq)
            .await?
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

    pub async fn load_legacy_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            pubsub::PubSub::Items(pubsub::pubsub::Items {
                max_items: None,
                node: NodeName(ns::BOOKMARKS.to_string()),
                subid: None,
                items: vec![],
            }),
        );

        let response = self
            .ctx
            .send_iq(iq)
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        let pubsub::PubSub::Items(items) = pubsub::PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        let Some(storage) = items.items.into_iter().find_map(|item| {
            let Some(payload) = &item.payload else {
                return None;
            };
            if !payload.is("storage", ns::BOOKMARKS) {
                return None;
            }
            return Some(bookmarks::Storage::try_from(payload.clone()))
        }) else {
            return Ok(vec![])
        };

        let bookmarks = storage?
            .conferences
            .into_iter()
            .map(ConferenceBookmark::from)
            .collect();

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

    pub async fn publish_legacy_bookmarks(
        &self,
        bookmarks: impl IntoIterator<Item = ConferenceBookmark>,
    ) -> Result<()> {
        let storage = bookmarks::Storage {
            conferences: bookmarks.into_iter().map(Into::into).collect(),
            urls: vec![],
        };

        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::BOOKMARKS.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: None,
                        publisher: None,
                        payload: Some(storage.into()),
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
            .schedule_event(ClientEvent::Bookmark(Event::BookmarksPublished {
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
            .schedule_event(ClientEvent::Bookmark(Event::BookmarksRetracted { jids }));
        Ok(())
    }

    fn handle_legacy_bookmarks(&self, items: &Vec<pubsub::event::Item>) -> Result<()> {
        let Some(storage) = items.iter().find_map(|item| {
            let Some(payload) = &item.payload else {
                return None;
            };
            if !payload.is("storage", ns::BOOKMARKS) {
                return None;
            }
            return Some(bookmarks::Storage::try_from(payload.clone()))
        }) else {
            return Ok(())
        };

        let bookmarks = storage?
            .conferences
            .into_iter()
            .map(ConferenceBookmark::from)
            .collect();

        self.ctx
            .schedule_event(ClientEvent::Bookmark(Event::BookmarksReplaced {
                bookmarks,
            }));

        Ok(())
    }
}
