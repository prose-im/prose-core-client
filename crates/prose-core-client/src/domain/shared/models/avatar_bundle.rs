// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::Avatar;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AvatarBundle {
    pub avatar: Option<Avatar>,
    pub initials: String,
    pub color: String,
}
