// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_settings::AccountSettings;
pub use availability::Availability;
pub use avatar_metadata::AvatarMetadata;
pub(crate) use bookmarks::Bookmarks;
pub use capabilities::{Capabilities, Feature};
pub use contact::Contact;
pub use message::{Emoji, Message, MessageId, Reaction, StanzaId};
pub use message_like::MessageLike;
pub use muc::Room;
pub use page::Page;
pub use presence::Presence;
pub use software_version::SoftwareVersion;
pub use user_activity::UserActivity;
pub use user_metadata::UserMetadata;
pub use user_profile::{Address, Url, UserProfile};

mod account_settings;
mod availability;
mod avatar_metadata;
mod bookmarks;
mod capabilities;
mod contact;
mod error;
mod message;
pub mod message_like;
pub(crate) mod muc;
mod page;
pub mod presence;
pub mod roster;
mod software_version;
mod user_activity;
pub mod user_metadata;
mod user_profile;
