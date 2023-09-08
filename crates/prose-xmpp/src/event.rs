// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{client, mods};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Bookmark(mods::bookmark::Event),
    Bookmark2(mods::bookmark2::Event),
    Caps(mods::caps::Event),
    Chat(mods::chat::Event),
    Client(client::Event),
    MUC(mods::muc::Event),
    Ping(mods::ping::Event),
    Profile(mods::profile::Event),
    Status(mods::status::Event),
}
