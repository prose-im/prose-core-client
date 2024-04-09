// prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use jid::BareJid;

use prose_core_client::dtos::{
    Bookmark, Contact, DeviceInfo, ParticipantInfo, PublicRoomInfo, RoomEnvelope, SidebarItem,
};
use prose_xmpp::mods::muc;

use crate::{ConnectedRoomExt, StringExt};

#[derive(Debug)]
pub struct JidWithName {
    pub jid: BareJid,
    pub name: String,
}

impl Display for JidWithName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<30} | {}", self.name.truncate_to(30), self.jid)
    }
}

impl From<RoomEnvelope> for JidWithName {
    fn from(value: RoomEnvelope) -> Self {
        Self {
            jid: value.to_generic_room().jid().clone().into_bare(),
            name: format!(
                "{} {}",
                value.kind(),
                value
                    .to_generic_room()
                    .name()
                    .unwrap_or("<untitled>".to_string())
            ),
        }
    }
}

impl From<muc::Room> for JidWithName {
    fn from(value: muc::Room) -> Self {
        Self {
            jid: value.jid.into_bare(),
            name: value.name.as_deref().unwrap_or("<untitled>").to_string(),
        }
    }
}

impl From<PublicRoomInfo> for JidWithName {
    fn from(value: PublicRoomInfo) -> Self {
        Self {
            jid: value.id.into_inner(),
            name: value.name.as_deref().unwrap_or("<untitled>").to_string(),
        }
    }
}

impl From<Contact> for JidWithName {
    fn from(value: Contact) -> Self {
        Self {
            jid: value.id.into_inner(),
            name: value.name,
        }
    }
}

impl From<SidebarItem> for JidWithName {
    fn from(value: SidebarItem) -> Self {
        Self {
            jid: value.room.to_generic_room().jid().clone().into_bare(),
            name: value.name,
        }
    }
}

impl From<Bookmark> for JidWithName {
    fn from(value: Bookmark) -> Self {
        Self {
            jid: value.jid.into_bare(),
            name: value.name,
        }
    }
}

pub struct ConnectedRoomEnvelope(pub RoomEnvelope);

impl Display for ConnectedRoomEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:<40} | {:<70} | {}",
            self.0.kind(),
            self.0
                .to_generic_room()
                .name()
                .unwrap_or("<untitled>".to_string())
                .truncate_to(40),
            self.0.to_generic_room().jid().to_string().truncate_to(70),
            self.0
                .to_generic_room()
                .subject()
                .as_deref()
                .unwrap_or("<no subject>")
        )
    }
}

pub struct ParticipantEnvelope(pub ParticipantInfo);

impl Display for ParticipantEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<20} {:<20} {:<10} {}",
            self.0
                .id
                .as_ref()
                .map(|jid| jid.to_string())
                .unwrap_or("<unknown real jid>".to_string())
                .truncate_to(20),
            self.0.name,
            self.0.affiliation,
            self.0.availability
        )
    }
}

pub struct DeviceInfoEnvelope(pub DeviceInfo);

impl Display for DeviceInfoEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>10} | {} | trusted: {} | {:<50}",
            if self.0.is_this_device { ">" } else { " " },
            self.0.id.as_ref(),
            self.0.fingerprint(),
            self.0.is_trusted,
            self.0
                .label
                .as_deref()
                .unwrap_or("<no label>")
                .to_string()
                .truncate_to(50),
        )
    }
}
