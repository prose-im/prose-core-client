// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_bookmark::AccountBookmark;
pub use attachment::Attachment;
pub use avatar::Avatar;
pub use client_event::ClientEvent;
pub use contact::{Availability, Contact, Group};
pub use errors::{ClientError, ClientResult, ConnectionError, JidParseError};
pub use jid::{parse_jid, JID};
pub use message::{Message, Reaction};
pub use message_result_set::MessageResultSet;
pub use participant_info::{ParticipantBasicInfo, ParticipantInfo};
pub use room::RoomEnvelope;
pub use send_message_request::SendMessageRequest;
pub use user_profile::UserProfile;

mod account_bookmark;
mod attachment;
mod avatar;
mod client_event;
mod contact;
mod errors;
mod jid;
mod message;
mod message_result_set;
mod participant_info;
mod room;
mod send_message_request;
mod upload_slot;
mod user_profile;
