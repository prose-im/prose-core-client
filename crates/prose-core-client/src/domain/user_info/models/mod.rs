// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use avatar_metadata::{AvatarInfo, AvatarMetadata};
pub use platform_image::PlatformImage;
pub use presence::Presence;
pub use user_activity::UserActivity;
pub use user_info::UserInfo;
pub use user_metadata::{LastActivity, UserMetadata};

mod avatar_metadata;
mod platform_image;
mod presence;
mod user_activity;
mod user_info;
mod user_metadata;
