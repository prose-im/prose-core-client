// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use std::str::FromStr;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::{bookmarks, pubsub};

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
            jid: Jid::from(conference.jid),
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
