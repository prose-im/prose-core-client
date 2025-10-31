// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::bookmarks::Storage;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub::NodeName;
use xmpp_parsers::{bookmarks, pubsub};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::ns;
use crate::stanza::ConferenceBookmark;
use crate::util::{PubSubItemsExt, PubSubQuery, PublishOptionsExt};
use crate::Event as ClientEvent;

/// XEP-0048: Bookmarks
/// https://xmpp.org/extensions/xep-0048.html
#[derive(Default, Clone)]
pub struct Bookmark {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    BookmarksChanged { bookmarks: Vec<ConferenceBookmark> },
}

impl Module for Bookmark {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_pubsub_event(&self, _from: &Jid, event: &pubsub::event::Payload) -> Result<()> {
        match event {
            pubsub::event::Payload::Items {
                node, published, ..
            } if node.0 == ns::BOOKMARKS && !published.is_empty() => {
                self.handle_changed_bookmarks(published)
            }
            _ => Ok(()),
        }
    }
}

impl Bookmark {
    pub async fn load_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let storage = self
            .ctx
            .query_pubsub_node(PubSubQuery::new(self.ctx.generate_id(), ns::BOOKMARKS))
            .await?
            .unwrap_or_default()
            .find_first_payload::<Storage>("storage", ns::BOOKMARKS)?
            .unwrap_or_default();

        let bookmarks = storage
            .conferences
            .into_iter()
            .map(ConferenceBookmark::from)
            .collect();

        Ok(bookmarks)
    }

    pub async fn publish_bookmarks(
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
                    items: vec![pubsub::pubsub::Item {
                        id: None,
                        publisher: None,
                        payload: Some(storage.into()),
                    }],
                },
                publish_options: Some(pubsub::pubsub::PublishOptions::for_private_data(None)),
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }
}

impl Bookmark {
    fn handle_changed_bookmarks(&self, items: &Vec<pubsub::event::Item>) -> Result<()> {
        let Some(storage) = items.iter().find_map(|item| {
            let Some(payload) = &item.payload else {
                return None;
            };
            if !payload.is("storage", ns::BOOKMARKS) {
                return None;
            }
            return Some(bookmarks::Storage::try_from(payload.clone()));
        }) else {
            return Ok(());
        };

        let bookmarks = storage?
            .conferences
            .into_iter()
            .map(ConferenceBookmark::from)
            .collect();

        self.ctx
            .schedule_event(ClientEvent::Bookmark(Event::BookmarksChanged { bookmarks }));

        Ok(())
    }
}
