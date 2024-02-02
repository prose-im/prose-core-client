// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use url::Url;

pub use account_info::AccountInfo;
pub use contact::{Contact, Group};
pub use message::{Message, MessageSender};
pub use presence_sub_request::{PresenceSubRequest, PresenceSubRequestId};
pub use sidebar_item::SidebarItem;

#[cfg(any(feature = "debug", feature = "test"))]
pub use crate::domain::sidebar::models::Bookmark;
pub use crate::domain::{
    contacts::models::PresenceSubscription,
    general::models::SoftwareVersion,
    messaging::models::{Emoji, MessageId, Reaction, StanzaId},
    rooms::models::{Participant, PublicRoomInfo, RoomAffiliation, RoomState},
    shared::models::{
        Availability, OccupantId, ParticipantId, ParticipantInfo, RoomId, UserBasicInfo, UserId,
        UserPresenceInfo, UserResourceId,
    },
    user_info::models::{LastActivity, UserInfo, UserMetadata, UserStatus},
    user_profiles::models::{Address, UserProfile},
};

mod account_info;
mod contact;
mod message;
mod presence_sub_request;
mod sidebar_item;
