// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::Avatar;
use crate::util::textual_palette::{
    generate_textual_initials, generate_textual_palette, normalize_textual_initials,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AvatarBundle {
    pub avatar: Option<Avatar>,
    pub initials: String,
    pub color: String,
}

impl AvatarBundle {
    pub fn with_generated_initials_and_color(
        id: &impl ToString,
        name: &str,
        avatar: Option<&Avatar>,
    ) -> Self {
        AvatarBundle {
            avatar: avatar.cloned(),
            initials: generate_textual_initials(name)
                .map(normalize_textual_initials)
                .unwrap_or_default(),
            color: generate_textual_palette(&id.to_string()),
        }
    }
}
