// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use jabber_client::{JabberClient, PROSE_IM_NODE};
pub use platform_image::PlatformImage;
pub use presence::Presence;
pub use user_info::{ProfileName, UserInfo, UserInfoOptExt, UserName};
pub use user_metadata::{LastActivity, UserMetadata};
pub use user_profile::{Address, Image, UserProfile};
pub use user_status::UserStatus;

mod jabber_client;
mod platform_image;
mod presence;
mod user_info;
mod user_metadata;
mod user_profile;
mod user_status;
