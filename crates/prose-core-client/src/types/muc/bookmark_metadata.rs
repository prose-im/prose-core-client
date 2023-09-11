// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::ns;
use anyhow::Error;
use jid::BareJid;
use minidom::Element;
use prose_xmpp::stanza::ConferenceBookmark;
use prose_xmpp::ElementExt;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Clone, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum RoomType {
    Group,
    PrivateChannel,
    PublicChannel,
}

pub struct BookmarkMetadata {
    pub room_type: RoomType,
    pub participants: Option<Vec<BareJid>>,
}

impl From<BookmarkMetadata> for Element {
    fn from(value: BookmarkMetadata) -> Self {
        Element::builder("metadata", ns::PROSE_BOOKMARK_METADATA)
            .append(
                Element::builder("type", ns::PROSE_BOOKMARK_METADATA)
                    .append(value.room_type.to_string()),
            )
            .append_all(value.participants.map(|participants| {
                Element::builder("participants", ns::PROSE_BOOKMARK_METADATA)
                    .append_all(participants)
            }))
            .build()
    }
}

impl TryFrom<Element> for BookmarkMetadata {
    type Error = Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("metadata", ns::PROSE_BOOKMARK_METADATA)?;

        Ok(BookmarkMetadata {
            room_type: value
                .get_child("type", ns::PROSE_BOOKMARK_METADATA)
                .ok_or(anyhow::format_err!(
                    "Missing element 'type' in BookmarkMetadata"
                ))
                .and_then(|child| RoomType::from_str(&child.text()).map_err(Error::from))?,
            participants: value
                .get_child("participants", ns::PROSE_BOOKMARK_METADATA)
                .map(|participants| {
                    participants
                        .children()
                        .map(|child| BareJid::from_str(&child.text()))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
        })
    }
}

pub trait BookmarkExt {
    fn prose_metadata(&self) -> Option<BookmarkMetadata>;
}

impl BookmarkExt for ConferenceBookmark {
    fn prose_metadata(&self) -> Option<BookmarkMetadata> {
        let Some(metadata_element) = self
            .conference
            .extensions
            .iter()
            .find(|elem| elem.is("metadata", ns::PROSE_BOOKMARK_METADATA))
        else {
            return None;
        };

        BookmarkMetadata::try_from(metadata_element.clone()).ok()
    }
}
