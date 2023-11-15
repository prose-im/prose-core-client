// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::muc::user::Status;
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::muc::RoomOccupancy;

use crate::domain::rooms::models::{RoomMetadata, RoomSettings};
use crate::dtos::RoomJid;
use crate::test::mock_app_dependencies::mock_account_jid;

impl RoomMetadata {
    pub fn new_room(room_jid: impl Into<RoomJid>) -> Self {
        Self {
            room_jid: room_jid
                .into()
                .into_inner()
                .with_resource_str(mock_account_jid().node_str().unwrap())
                .unwrap(),
            occupancy: RoomOccupancy {
                user: MucUser {
                    status: vec![Status::RoomHasBeenCreated],
                    items: vec![],
                },
                self_presence: Presence {
                    from: None,
                    to: None,
                    id: None,
                    type_: Default::default(),
                    show: None,
                    statuses: Default::default(),
                    priority: 0,
                    payloads: vec![],
                },
                presences: vec![],
            },
            settings: RoomSettings {
                features: Default::default(),
                name: None,
                description: None,
            },
            members: vec![],
        }
    }

    pub fn with_public_channel_features(mut self) -> Self {
        self.settings.features = Default::default();
        self.settings.features.is_persistent = true;
        self.settings.features.is_public = true;
        self.settings.features.is_open = true;
        self
    }
}
