// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkType {
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
}
