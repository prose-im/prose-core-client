// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::ConnectionError;

use crate::domain::{
    rooms::models::{ComposeState, RoomAffiliation},
    shared::models::{Availability, OccupantId, RoomId, UserEndpointId, UserId, UserResourceId},
    user_info::models::{AvatarMetadata, UserStatus},
    user_profiles::models::UserProfile,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    Connection(ConnectionEvent),
    UserStatus(UserStatusEvent),
    UserInfo(UserInfoEvent),
    Room(RoomEvent),
    Occupant(OccupantEvent),
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
    pub r#type: UserDataEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserDataEventType {
    StatusChanged { status: UserStatus },
    ProfileChanged { profile: UserProfile },
    AvatarChanged { metadata: AvatarMetadata },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestEvent {
    pub request_id: String,
    // TODO: From / which id type?
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestEventType {
    Ping,
    Capabilities,
    LocalTime,
    SoftwareVersion,
    LastActivity,
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
    /// The global id of the occupant on their server.
    pub real_id: Option<UserResourceId>,
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
