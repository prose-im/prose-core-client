// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_info::AccountInfo;
pub use attachment::{Attachment, Thumbnail};
pub use channel::{Channel, ChannelsArray};
pub use connection_error::{ConnectionError, ConnectionErrorType};
pub use contact::{Availability, Contact, UserStatus};
pub use jid::{BareJid, ParticipantId};
pub use js_array::*;
pub use mention::Mention;
pub use message::Message;
pub use message_result_set::MessageResultSet;
pub use presence_sub_request::{PresenceSubRequest, PresenceSubRequestArray, PresenceSubRequestId};
pub use room::RoomEnvelopeExt;
pub use send_message_request::SendMessageRequest;
pub use sidebar_item::{SidebarItem, SidebarItemsArray};
pub use upload_slot::UploadSlot;
pub use user_info::{
    Avatar, ParticipantBasicInfo, ParticipantBasicInfoArray, ParticipantInfo, ParticipantInfoArray,
    UserBasicInfo, UserBasicInfoArray,
};
pub use user_metadata::UserMetadata;
pub use user_profile::UserProfile;

mod account_info;
mod attachment;
mod channel;
mod connection_error;
mod contact;
mod jid;
mod js_array;
mod mention;
mod message;
mod message_result_set;
mod presence_sub_request;
mod room;
mod send_message_request;
mod sidebar_item;
mod upload_slot;
mod user_info;
mod user_metadata;
mod user_profile;
