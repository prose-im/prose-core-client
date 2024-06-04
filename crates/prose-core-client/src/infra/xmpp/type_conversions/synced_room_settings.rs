// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use minidom::Element;
use xmpp_parsers::pubsub::PubSubPayload;

use prose_xmpp::{ElementExt, ParseError};

use crate::domain::settings::models::SyncedRoomSettings;
use crate::dtos::RoomId;
use crate::infra::xmpp::type_conversions::message_ref;

pub mod ns {
    pub const PROSE_ROOM_SETTINGS: &str = "https://prose.org/protocol/room_settings";
}

impl TryFrom<Element> for SyncedRoomSettings {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Self {
            room_id: value.attr_req("room-id")?.parse().map_err(
                |err: <RoomId as FromStr>::Err| ParseError::Generic {
                    msg: err.to_string(),
                },
            )?,
            last_read_message: value
                .get_child("message-ref", message_ref::ns::PROSE_MESSAGE_REF)
                .cloned()
                .map(TryFrom::try_from)
                .transpose()?,
            encryption_enabled: value
                .get_child("encryption", ns::PROSE_ROOM_SETTINGS)
                .map(|child| child.attr_req("type"))
                .transpose()?
                == Some("omemo"),
        })
    }
}

impl From<SyncedRoomSettings> for Element {
    fn from(value: SyncedRoomSettings) -> Self {
        Element::builder("room-settings", ns::PROSE_ROOM_SETTINGS)
            .attr("room-id", value.room_id.to_raw_key_string())
            .append_all(value.last_read_message)
            .append(
                Element::builder("encryption", ns::PROSE_ROOM_SETTINGS).attr(
                    "type",
                    if value.encryption_enabled {
                        "omemo"
                    } else {
                        "none"
                    },
                ),
            )
            .build()
    }
}

impl PubSubPayload for SyncedRoomSettings {}
