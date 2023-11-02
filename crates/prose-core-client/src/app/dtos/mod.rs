// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use url::Url;

pub use contact::Contact;
pub use message::{Message, MessageSender};

pub use crate::domain::{
    contacts::models::Group,
    general::models::SoftwareVersion,
    messaging::models::{Emoji, MessageId, Reaction, StanzaId},
    rooms::models::{Member, Occupant},
    shared::models::{Availability, UserBasicInfo, UserPresenceInfo},
    user_info::models::{LastActivity, UserActivity, UserInfo, UserMetadata},
    user_profiles::models::{Address, UserProfile},
};

mod contact;
mod message;
