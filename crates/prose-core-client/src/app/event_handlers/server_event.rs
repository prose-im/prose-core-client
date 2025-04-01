// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use prose_xmpp::ConnectionError;

use crate::domain::contacts::models::PresenceSubscription;
use crate::domain::encryption::models::DeviceList;
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::MucId;
use crate::domain::sidebar::models::Bookmark;
use crate::domain::user_info::models::Presence;
use crate::domain::{
    rooms::models::{ComposeState, RoomAffiliation},
    shared::models::{
        AnonOccupantId, AvatarMetadata, CapabilitiesId, OccupantId, RequestId, SenderId,
        UserEndpointId, UserId, UserResourceId,
    },
    user_info::models::{UserProfile, UserStatus},
};
use crate::dtos::RoomId;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    /// Events about modifications to the block list.
    BlockList(BlockListEvent),
    /// Event related to the connection status.
    Connection(ConnectionEvent),
    /// Events that are related to contacts.
    ContactList(ContactListEvent),
    /// Events that affect the status of a user within a conversation or globally.
    UserStatus(UserStatusEvent),
    /// Events that affect the information about the user globally.
    UserInfo(UserInfoEvent),
    /// Events about changes to a MUC room.
    Room(RoomEvent),
    /// Events about changes to an occupant of a MUC room.
    Occupant(OccupantEvent),
    /// Events about requests that are directed at us.
    Request(RequestEvent),
    /// Events about received messages.
    Message(MessageEvent),
    /// Events about changes to the sidebar.
    SidebarBookmark(SidebarBookmarkEvent),
    /// Events about RoomSettings that are shared between our clients.
    SyncedRoomSettings(SyncedRoomSettingsEvent),
    /// Events about OMEMO devices.
    UserDevice(UserDeviceEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    Connected,
    Disconnected { error: Option<ConnectionError> },
    PingTimer,
}

#[derive(Debug, Clone, PartialEq)]
// Events that affect the status of a user within a conversation or globally.
pub struct UserStatusEvent {
    pub user_id: UserEndpointId,
    pub r#type: UserStatusEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatusEventType {
    PresenceChanged { presence: Presence },
    ComposeStateChanged { state: ComposeState },
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
    StatusChanged { status: Option<UserStatus> },
    NicknameChanged { nickname: Option<String> },
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
    pub room_id: MucId,
    pub r#type: RoomEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomEventType {
    /// The room was destroyed and potentially replaced by `replacement`.
    Destroyed { replacement: Option<MucId> },
    /// The configuration _or_ name of the room was changed.
    RoomConfigChanged,
    /// The topic of the room was changed.
    RoomTopicChanged { new_topic: Option<String> },
    /// `sender` sent you an invitation to this room.
    ReceivedInvitation {
        sender: UserResourceId,
        password: Option<String>,
    },
    /// A user was added via an invitation.
    UserAdded {
        user_id: UserId,
        affiliation: RoomAffiliation,
        reason: Option<String>,
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
    Received(prose_xmpp::stanza::Message),
    Sync(prose_xmpp::mods::chat::Carbon),
    Sent(prose_xmpp::stanza::Message),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubSubEvent<Id, Item> {
    pub user_id: UserId,
    pub r#type: PubSubEventType<Id, Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PubSubEventType<Id, Item> {
    AddedOrUpdated { items: Vec<Item> },
    Deleted { ids: Vec<Id> },
    Purged,
}

pub type SidebarBookmarkEvent = PubSubEvent<BareJid, Bookmark>;
pub type UserDeviceEvent = PubSubEvent<String, DeviceList>;
pub type SyncedRoomSettingsEvent = PubSubEvent<RoomId, SyncedRoomSettings>;

#[derive(Debug, Clone, PartialEq)]
pub enum ContactListEventType {
    /// The contact was either added to our contact list or the presence subscription to or from
    /// the contact changed.
    ContactAddedOrPresenceSubscriptionUpdated { subscription: PresenceSubscription },
    /// The contact was removed from our contact list.
    ContactRemoved,
    /// The contact requested to subscribe to our presence.
    PresenceSubscriptionRequested { nickname: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContactListEvent {
    pub contact_id: UserId,
    pub r#type: ContactListEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockListEventType {
    UserBlocked { user_id: UserId },
    UserUnblocked { user_id: UserId },
    BlockListCleared,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockListEvent {
    pub r#type: BlockListEventType,
}
