// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use anon_occupant_id::AnonOccupantId;
pub use availability::Availability;
pub use capabilities_id::CapabilitiesId;
pub use occupant_id::OccupantId;
pub use participant_id::ParticipantId;
pub use request_id::RequestId;
pub use room_id::RoomId;
pub use room_type::RoomType;
pub use sender_id::SenderId;
pub use user_endpoint_id::UserEndpointId;
pub use user_id::UserId;
pub use user_info::{UserBasicInfo, UserPresenceInfo};
pub use user_or_resource_id::UserOrResourceId;
pub use user_resource_id::UserResourceId;

mod anon_occupant_id;
mod availability;
mod capabilities_id;
mod occupant_id;
mod participant_id;
mod request_id;
mod room_id;
mod room_type;
mod sender_id;
mod user_endpoint_id;
mod user_id;
mod user_info;
mod user_or_resource_id;
mod user_resource_id;
