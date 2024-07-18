// prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use jid::BareJid;

use prose_core_client::dtos::{
    Avatar, AvatarSource, Bookmark, Contact, DeviceInfo, DeviceTrust, Message, ParticipantInfo,
    PublicRoomInfo, RoomEnvelope, SidebarItem, UserBasicInfo,
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
            "{id:<60} {real_id:<20} {name:<20} {aff:<10} {avatar} {avail}",
            id = self.0.id.to_string().truncate_to(60),
            real_id = self
                .0
                .user_id
                .as_ref()
                .map(|jid| format!("({})", jid.to_string().truncate_to(20)))
                .unwrap_or("(<unknown real jid>)".to_string()),
            name = self.0.name.truncate_to(20),
            aff = self.0.affiliation,
            avatar = self
                .0
                .avatar
                .as_ref()
                .map(|avatar| AvatarEnvelope(avatar.clone()).to_string())
                .unwrap_or_else(|| "<no avatar>".to_string()),
            avail = self.0.availability
        )
    }
}

pub struct AvatarEnvelope(pub Avatar);

impl Display for AvatarEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.0.source {
            AvatarSource::Pep { .. } => "PEP",
            AvatarSource::Vcard { .. } => "vCard",
        };
        write!(f, "{id} ({kind})", id = self.0.id)
    }
}

pub struct DeviceInfoEnvelope(pub DeviceInfo);

impl Display for DeviceInfoEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let trust = match self.0.trust {
            DeviceTrust::Trusted => "trusted",
            DeviceTrust::Untrusted => "untrusted",
            DeviceTrust::Undecided => "undecided",
            DeviceTrust::Verified => "verified",
        };

        write!(
            f,
            "{} {:>10} | {} | trust: {}",
            if self.0.is_this_device { ">" } else { " " },
            self.0.id.as_ref(),
            self.0.fingerprint(),
            trust,
        )
    }
}

pub struct UserBasicInfoEnvelope(pub UserBasicInfo);

impl Display for UserBasicInfoEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.0.name, self.0.id)
    }
}

pub struct MessageEnvelope(pub Message);

impl Display for MessageEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reactions = self
            .0
            .reactions
            .iter()
            .map(|reaction| {
                let senders = reaction
                    .from
                    .iter()
                    .map(|sender| format!("{} ({})", sender.name, sender.id.to_opaque_identifier()))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{}: {}", reaction.emoji, senders)
            })
            .collect::<Vec<_>>()
            .join(" | ");

        write!(
            f,
            "{} | {:<36} | {:<20} | {} attachments | {} mentions | {}{}",
            self.0.timestamp.format("%Y/%m/%d %H:%M:%S"),
            self.0
                .id
                .as_ref()
                .map(|id| id.clone().into_inner())
                .unwrap_or("<no-id>".to_string()),
            self.0.from.id.to_opaque_identifier().truncate_to(20),
            self.0.attachments.len(),
            self.0.mentions.len(),
            self.0.body.html,
            if self.0.reactions.is_empty() {
                "".to_string()
            } else {
                format!("\n{}", reactions)
            }
        )
    }
}

pub struct CompactMessageEnvelope(pub Message);

impl Display for CompactMessageEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {:<36} | {:<20} | {}",
            self.0.timestamp.format("%Y/%m/%d %H:%M:%S"),
            self.0
                .id
                .as_ref()
                .map(|id| id.clone().into_inner())
                .unwrap_or("<no-id>".to_string()),
            self.0.from.id.to_opaque_identifier().truncate_to(20),
            self.0.body.html.to_string().truncate_to(40),
        )
    }
}
