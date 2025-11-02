// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_bookmark::AccountBookmark;
pub use account_info::AccountInfo;
pub use attachment::Attachment;
pub use avatar::Avatar;
pub use client_event::ClientEvent;
pub use contact::{Availability, Contact, Group, UserStatus};
pub use errors::{ClientError, ClientResult, ConnectionError};
pub use message::{Message, Reaction};
pub use message_result_set::MessageResultSet;
pub use participant_info::{ParticipantBasicInfo, ParticipantInfo};
pub use presence_sub_request::PresenceSubRequest;
pub use public_room_info::PublicRoomInfo;
pub use room::{RoomEnvelope, RoomState};
pub use send_message_request::SendMessageRequest;
pub use sidebar_item::SidebarItem;
pub use upload_slot::UploadSlot;
pub use user_info::UserBasicInfo;
pub use user_metadata::UserMetadata;
pub use user_profile::UserProfile;
pub use workspace_info::{WorkspaceIcon, WorkspaceInfo};

mod account_bookmark;
mod account_info;
mod attachment;
mod avatar;
mod client_event;
mod contact;
mod errors;
mod message;
mod message_result_set;
mod participant_info;
mod presence_sub_request;
mod public_room_info;
mod room;
mod send_message_request;
mod sidebar_item;
mod upload_slot;
mod user_info;
mod user_metadata;
mod user_profile;
mod workspace_info;
