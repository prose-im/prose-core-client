// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkType {
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
}

impl Display for BookmarkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BookmarkType::DirectMessage => "Direct Message",
                BookmarkType::Group => "Group",
                BookmarkType::PrivateChannel => "Private Channel",
                BookmarkType::PublicChannel => "Public Channel",
            }
        )
    }
}
