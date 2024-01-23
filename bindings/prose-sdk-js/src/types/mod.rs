// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_info::AccountInfo;
pub use channel::{Channel, ChannelsArray};
pub use contact::{Availability, Contact};
pub use jid::BareJid;
pub use js_array::*;
pub use message::Message;
pub use room::RoomEnvelopeExt;
pub use sidebar_item::{SidebarItem, SidebarItemsArray};
pub use user_info::{ParticipantInfo, ParticipantInfoArray, UserBasicInfo, UserBasicInfoArray};
pub use user_metadata::UserMetadata;
pub use user_profile::UserProfile;

mod account_info;
mod channel;
mod contact;
mod jid;
mod js_array;
mod message;
mod room;
mod sidebar_item;
mod user_info;
mod user_metadata;
mod user_profile;
