// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::caps::Caps;
use xmpp_parsers::nick::Nick;
use xmpp_parsers::presence;
use xmpp_parsers::vcard_update::VCardUpdate;

use prose_xmpp::ns;
use prose_xmpp::stanza::muc::MucUser;

use crate::domain::shared::models::{AnonOccupantId, Availability, AvatarId, ParticipantId};
use crate::domain::user_info::models::Presence;
use crate::dtos::{Avatar, AvatarSource};
use crate::infra::xmpp::util::CapsExt;

pub trait PresenceExt {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId>;
    fn availability(&self) -> Availability;
    fn avatar_id(&self) -> Option<AvatarId>;
    fn nickname(&self) -> Option<String>;
    fn caps(&self) -> Option<Caps>;
    fn muc_user(&self) -> Option<MucUser>;

    fn to_domain_presence(&self, participant: impl Into<ParticipantId>) -> Presence;
}

impl PresenceExt for presence::Presence {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId> {
        self.payloads
            .iter()
            .find(|p| p.is("occupant-id", ns::OCCUPANT_ID))
            .and_then(|e| e.attr("id"))
            .map(|id| AnonOccupantId::from(id.to_string()))
    }

    fn availability(&self) -> Availability {
        Availability::from((
            (self.type_ != presence::Type::None).then_some(self.type_.clone()),
            self.show.clone(),
        ))
    }

    fn avatar_id(&self) -> Option<AvatarId> {
        self.payloads
            .iter()
            .find(|p| p.is("x", ns::VCARD_UPDATE))
            .cloned()
            .and_then(|p| VCardUpdate::try_from(p).ok())
            .and_then(|vcard| vcard.photo)
            .and_then(|photo| photo.data)
            .map(|sha1_bytes| {
                let mut sha1_str = String::with_capacity(40);
                for byte in sha1_bytes {
                    sha1_str.extend(format!("{:02x}", byte).chars());
                }
                AvatarId::from_str_unchecked(sha1_str)
            })
    }

    fn nickname(&self) -> Option<String> {
        self.payloads
            .iter()
            .find(|p| p.is("nick", ns::NICK))
            .cloned()
            .and_then(|p| Nick::try_from(p).ok())
            .map(|nick| nick.0)
    }

    fn caps(&self) -> Option<Caps> {
        self.payloads
            .iter()
            .find(|p| p.is("c", ns::CAPS))
            .cloned()
            .and_then(|p| Caps::try_from(p).ok())
    }

    fn muc_user(&self) -> Option<MucUser> {
        self.payloads
            .iter()
            .find(|p| p.is("x", ns::MUC_USER))
            .cloned()
            .and_then(|p| MucUser::try_from(p).ok())
    }

    fn to_domain_presence(&self, participant: impl Into<ParticipantId>) -> Presence {
        let avatar = self.avatar_id().map(|avatar_id| Avatar {
            id: avatar_id,
            source: AvatarSource::Vcard,
            owner: participant.into(),
        });

        let (caps, client) = self
            .caps()
            .map(|caps| (Some(caps.id()), caps.client()))
            .unwrap_or_default();

        Presence {
            priority: self.priority,
            availability: self.availability(),
            avatar,
            caps,
            client,
            status: self.statuses.first_key_value().map(|v| v.1.clone()),
            nickname: self.nickname(),
        }
    }
}
