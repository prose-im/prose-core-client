use jid::BareJid;
use prose_xmpp::stanza::ConferenceBookmark;
use std::collections::hash_map::Values;
use std::collections::{HashMap, HashSet};
use std::iter::Chain;

#[derive(Default)]
pub(crate) struct Bookmarks {
    pub bookmarks: HashMap<BareJid, ConferenceBookmark>,
    pub bookmarks2: HashMap<BareJid, ConferenceBookmark>,
}

impl Bookmarks {
    pub fn new(bookmarks: Vec<ConferenceBookmark>, bookmarks2: Vec<ConferenceBookmark>) -> Self {
        Self {
            bookmarks: bookmarks
                .into_iter()
                .map(|bookmark| (bookmark.jid.to_bare(), bookmark))
                .collect(),
            bookmarks2: bookmarks2
                .into_iter()
                .map(|bookmark| (bookmark.jid.to_bare(), bookmark))
                .collect(),
        }
    }
}

impl Bookmarks {
    pub fn iter(&self) -> UniqueBookmarksIterator<'_> {
        UniqueBookmarksIterator {
            iter: self.bookmarks.values().chain(self.bookmarks2.values()),
            visited_jids: Default::default(),
        }
    }
}

pub struct UniqueBookmarksIterator<'a> {
    iter: Chain<Values<'a, BareJid, ConferenceBookmark>, Values<'a, BareJid, ConferenceBookmark>>,
    visited_jids: HashSet<BareJid>,
}

impl<'a> Iterator for UniqueBookmarksIterator<'a> {
    type Item = &'a ConferenceBookmark;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next) = self.iter.next() else {
            return None;
        };
        let bare_jid = next.jid.to_bare();
        if self.visited_jids.contains(&bare_jid) {
            return self.next();
        }
        self.visited_jids.insert(bare_jid);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prose_xmpp::jid_str;

    #[test]
    fn test_iterates_unique_bookmarks() {
        let bookmarks = Bookmarks::new(
            vec![
                ConferenceBookmark {
                    jid: jid_str!("a@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("b@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("c@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("e@prose.org"),
                    conference: Default::default(),
                },
            ],
            vec![
                ConferenceBookmark {
                    jid: jid_str!("a@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("b@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("d@prose.org"),
                    conference: Default::default(),
                },
                ConferenceBookmark {
                    jid: jid_str!("f@prose.org"),
                    conference: Default::default(),
                },
            ],
        );

        let iterated_bookmarks_jids = bookmarks
            .iter()
            .map(|bookmark| bookmark.jid.to_bare())
            .collect::<HashSet<_>>();

        assert_eq!(
            iterated_bookmarks_jids,
            HashSet::from_iter(
                [
                    jid_str!("a@prose.org").into_bare(),
                    jid_str!("b@prose.org").into_bare(),
                    jid_str!("c@prose.org").into_bare(),
                    jid_str!("e@prose.org").into_bare(),
                    jid_str!("d@prose.org").into_bare(),
                    jid_str!("f@prose.org").into_bare()
                ]
                .into_iter()
            )
        );
    }
}
