// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::HexColor;
use prose_core_client::dtos::{Avatar as CoreAvatar, AvatarBundle as CoreAvatarBundle};
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct Avatar(CoreAvatar);

#[uniffi::export]
impl Avatar {
    /// An opaque identifier to check if the contents of the `Avatar` have changed.
    /// While `ProseClient` caches loaded avatars, checking for a change in the `Avatar` might
    /// still make sense, since `Client::loadAvatarDataURL` is asynchronous.
    pub fn id(&self) -> String {
        format!("{}-{}", self.0.owner(), self.0.id)
    }
}

impl From<CoreAvatar> for Avatar {
    fn from(avatar: CoreAvatar) -> Self {
        Self(avatar)
    }
}

impl AsRef<CoreAvatar> for Avatar {
    fn as_ref(&self) -> &CoreAvatar {
        &self.0
    }
}

#[derive(uniffi::Record)]
pub struct AvatarBundle {
    pub avatar: Option<Arc<Avatar>>,
    pub initials: String,
    pub color: HexColor,
}

impl From<CoreAvatarBundle> for AvatarBundle {
    fn from(value: CoreAvatarBundle) -> Self {
        Self {
            avatar: value.avatar.map(|avatar| Arc::new(avatar.into())),
            initials: value.initials,
            color: value.color.into(),
        }
    }
}
