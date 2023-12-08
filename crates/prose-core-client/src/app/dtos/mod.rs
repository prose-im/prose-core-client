// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use url::Url;

pub use contact::Contact;
pub use message::{Message, MessageSender};
pub use sidebar_item::SidebarItem;

pub use crate::domain::{
    contacts::models::Group,
    general::models::SoftwareVersion,
    messaging::models::{Emoji, MessageId, Reaction, StanzaId},
    rooms::models::{Participant, PublicRoomInfo, RoomAffiliation},
    shared::models::{
        Availability, OccupantId, ParticipantInfo, RoomId, UserBasicInfo, UserId, UserPresenceInfo,
        UserResourceId,
    },
    user_info::models::{LastActivity, UserInfo, UserMetadata, UserStatus},
    user_profiles::models::{Address, UserProfile},
};

#[cfg(feature = "debug")]
pub use crate::domain::sidebar::models::Bookmark;

mod contact;
mod message;
mod sidebar_item;
