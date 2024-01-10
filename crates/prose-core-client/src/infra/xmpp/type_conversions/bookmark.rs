// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use jid::BareJid;
use minidom::{Element, IntoAttributeValue};
use xmpp_parsers::pubsub::PubSubPayload;

use crate::domain::rooms::models::RoomSidebarState;
use prose_xmpp::{ElementExt, ParseError, RequestError};

use crate::domain::shared::models::RoomId;
use crate::domain::sidebar::models::{Bookmark, BookmarkType};

pub mod ns {
    pub const PROSE_BOOKMARK: &str = "https://prose.org/protocol/bookmark";
}

impl TryFrom<Element> for Bookmark {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("bookmark", ns::PROSE_BOOKMARK)?;

        let in_sidebar = value.attr("sidebar").is_some();
        let is_favorite = value.attr("favorite").is_some();

        let sidebar_state = match (in_sidebar, is_favorite) {
            (true, true) => RoomSidebarState::Favorite,
            (true, _) => RoomSidebarState::InSidebar,
            (false, _) => RoomSidebarState::NotInSidebar,
        };

        Ok(Self {
            name: value.attr_req("name")?.to_string(),
            jid: RoomId::from(BareJid::from_str(&value.attr_req("jid")?)?),
            r#type: BookmarkType::from_str(&value.attr_req("type")?)?,
            sidebar_state,
        })
    }
}

impl From<Bookmark> for Element {
    fn from(value: Bookmark) -> Self {
        Element::builder("bookmark", ns::PROSE_BOOKMARK)
            .attr("name", value.name)
            .attr("jid", value.jid)
            .attr("type", value.r#type)
            .attr(
                "favorite",
                (value.sidebar_state == RoomSidebarState::Favorite).then_some("1"),
            )
            .attr(
                "sidebar",
                value.sidebar_state.is_in_sidebar().then_some("1"),
            )
            .build()
    }
}

impl PubSubPayload for Bookmark {}

impl IntoAttributeValue for BookmarkType {
    fn into_attribute_value(self) -> Option<String> {
        Some(
            match self {
                BookmarkType::DirectMessage => "dm",
                BookmarkType::Group => "group",
                BookmarkType::PrivateChannel => "private-channel",
                BookmarkType::PublicChannel => "public-channel",
            }
            .to_string(),
        )
    }
}

impl FromStr for BookmarkType {
    type Err = RequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dm" => Ok(Self::DirectMessage),
            "group" => Ok(Self::Group),
            "private-channel" => Ok(Self::PrivateChannel),
            "public-channel" => Ok(Self::PublicChannel),
            _ => Err(RequestError::ParseError(ParseError::Generic {
                msg: format!("Unknown RoomType {}", s),
            })),
        }
    }
}
