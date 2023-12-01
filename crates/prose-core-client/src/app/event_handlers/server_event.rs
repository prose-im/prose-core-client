// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::ConnectionError;

use crate::domain::shared::models::AnonOccupantId;
use crate::domain::shared::models::SenderId;
use crate::domain::shared::models::{CapabilitiesId, RequestId};
use crate::domain::{
    rooms::models::{ComposeState, RoomAffiliation},
    shared::models::{Availability, OccupantId, RoomId, UserEndpointId, UserId, UserResourceId},
    user_info::models::{AvatarMetadata, UserStatus},
    user_profiles::models::UserProfile,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    Connection(ConnectionEvent),
    /// Events that affect the status of a user within a conversation or globally.
    UserStatus(UserStatusEvent),
    /// Events that affect the information about the user globally.
    UserInfo(UserInfoEvent),
    /// Events that affect a specific resource of a user.
    UserResource(UserResourceEvent),
    /// Events about changes to a MUC room.
    Room(RoomEvent),
    /// Events about changes to an occupant of a MUC room.
    Occupant(OccupantEvent),
    /// Events about requests that are directed at us.
    Request(RequestEvent),
    // TODO…
    Message(MessageEvent),
    // TODO: Bookmarks (PubSub!)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    Connected,
    Disconnected { error: Option<ConnectionError> },
}

#[derive(Debug, Clone, PartialEq)]
// Events that affect the status of a user within a conversation or globally.
pub struct UserStatusEvent {
    pub user_id: UserEndpointId,
    pub r#type: UserStatusEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatusEventType {
    AvailabilityChanged { availability: Availability }, // TODO: Room/Full/Bare at initial presence
    ComposeStateChanged { state: ComposeState },        // TODO: Room/Full
}

#[derive(Debug, Clone, PartialEq)]
// Events that affect the information about the user globally.
pub struct UserInfoEvent {
    pub user_id: UserId,
    pub r#type: UserInfoEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserInfoEventType {
    AvatarChanged { metadata: AvatarMetadata },
    ProfileChanged { profile: UserProfile },
    StatusChanged { status: UserStatus },
}

#[derive(Debug, Clone, PartialEq)]
// Events that affect a specific resource of a user.
pub struct UserResourceEvent {
    pub user_id: UserResourceId,
    pub r#type: UserResourceEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserResourceEventType {
    CapabilitiesChanged { id: CapabilitiesId },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestEvent {
    pub sender_id: SenderId,
    pub request_id: RequestId,
    pub r#type: RequestEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestEventType {
    Ping,
    LocalTime,
    LastActivity,
    Capabilities { id: CapabilitiesId },
    SoftwareVersion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomEvent {
    pub room_id: RoomId,
    pub r#type: RoomEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomEventType {
    /// The room was destroyed and potentially replaced by `replacement`.
    Destroyed { replacement: Option<RoomId> },
    /// The configuration _or_ name of the room was changed.
    RoomConfigChanged,
    /// The topic of the room was changed.
    RoomTopicChanged { new_topic: Option<String> },
    /// `sender` sent you an invitation to this room.
    ReceivedInvitation {
        sender: UserResourceId,
        password: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OccupantEvent {
    /// The occupant's id within the room.
    pub occupant_id: OccupantId,
    /// The occupant's anonymous id (https://xmpp.org/extensions/xep-0421.html)
    pub anon_occupant_id: Option<AnonOccupantId>,
    /// The global id of the occupant on their server.
    pub real_id: Option<UserId>,
    /// Is this the current (logged-in) user?
    pub is_self: bool,
    /// The type of this event.
    pub r#type: OccupantEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OccupantEventType {
    /// The occupant's affiliation was modified.
    AffiliationChanged { affiliation: RoomAffiliation },
    /// The occupant was disconnected temporarily by the server, i.e. because of a restart.
    DisconnectedByServer,
    /// The occupant was permanently removed/banned from the room.
    PermanentlyRemoved,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageEvent {
    pub r#type: MessageEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEventType {
    Received, // Regular messages
    Sync,     // Carbons
}
