// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use url::Url;

pub use account_info::AccountInfo;
pub use contact::{Contact, Group};
pub use message::{Message, MessageSender, Reaction};
pub use message_result_set::MessageResultSet;
pub use presence_sub_request::{PresenceSubRequest, PresenceSubRequestId};
pub use room_envelope::RoomEnvelope;
pub use send_message_request::{Body as SendMessageRequestBody, SendMessageRequest};
pub use sidebar_item::SidebarItem;
pub use upload_slot::UploadSlot;

#[cfg(any(feature = "debug", feature = "test"))]
pub use crate::domain::sidebar::models::Bookmark;
pub use crate::domain::{
    contacts::models::PresenceSubscription,
    encryption::models::{
        DeviceBundle, DeviceId, DeviceInfo, DeviceTrust, IdentityKey, IdentityKeyPair,
        LocalEncryptionBundle, PreKeyBundle, PreKeyId, PreKeyRecord, PrivateKey, PublicKey,
        SessionData, SignedPreKeyId, SignedPreKeyRecord,
    },
    general::models::SoftwareVersion,
    messaging::models::{
        Attachment, AttachmentType, Emoji, EncryptedPayload, EncryptionKey, Mention, MessageId,
        StanzaId, Thumbnail,
    },
    rooms::models::{Participant, PublicRoomInfo, RoomAffiliation, RoomState},
    shared::models::{
        Availability, MucId, OccupantId, ParticipantId, ParticipantInfo, RoomId, ScalarRangeExt,
        StringIndexRangeExt, UnicodeScalarIndex, UserBasicInfo, UserId, UserPresenceInfo,
        UserResourceId, Utf16Index, Utf8Index,
    },
    uploads::models::UploadHeader,
    user_info::models::{LastActivity, UserInfo, UserMetadata, UserStatus},
    user_profiles::models::{Address, UserProfile},
};

mod account_info;
mod contact;
mod message;
mod message_result_set;
mod presence_sub_request;
mod room_envelope;
mod send_message_request;
mod sidebar_item;
mod upload_slot;
