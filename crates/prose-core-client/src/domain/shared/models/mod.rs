// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_id::AccountId;
pub use anon_occupant_id::AnonOccupantId;
pub use availability::Availability;
pub use avatar::{Avatar, AvatarSource};
pub use avatar_bundle::AvatarBundle;
pub use avatar_id::AvatarId;
pub use avatar_metadata::{AvatarInfo, AvatarMetadata};
pub use bare_entity_id::BareEntityId;
pub use cache_policy::CachePolicy;
pub use capabilities_id::CapabilitiesId;
pub use connection_state::ConnectionState;
pub use entity_id::{EntityId, EntityIdRef};
pub use mam_version::MamVersion;
pub use message::{Markdown, StyledMessage, HTML};
pub use muc_id::MucId;
pub use occupant_id::OccupantId;
pub use participant_id::{ParticipantId, ParticipantIdRef};
pub use request_id::RequestId;
pub use room_id::RoomId;
pub use room_type::RoomType;
pub use sender_id::SenderId;
pub use server_id::ServerId;
pub use string_index::{
    RustStringRangeExt, ScalarRangeExt, StringIndexRangeExt, UnicodeScalarIndex, Utf16Index,
    Utf8Index,
};
pub use user_endpoint_id::UserEndpointId;
pub use user_id::UserId;
pub use user_info::{ParticipantBasicInfo, ParticipantInfo, UserBasicInfo, UserPresenceInfo};
pub use user_or_resource_id::UserOrResourceId;
pub use user_resource_id::UserResourceId;

mod account_id;
mod anon_occupant_id;
mod availability;
mod avatar;
mod avatar_bundle;
mod avatar_id;
mod avatar_metadata;
mod bare_entity_id;
mod cache_policy;
mod capabilities_id;
mod connection_state;
mod entity_id;
mod mam_version;
mod message;
mod muc_id;
mod occupant_id;
mod participant_id;
mod request_id;
mod room_id;
mod room_type;
mod sender_id;
mod server_id;
mod string_index;
mod user_endpoint_id;
mod user_id;
mod user_info;
mod user_or_resource_id;
mod user_resource_id;
