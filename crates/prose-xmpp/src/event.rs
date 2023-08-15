// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{client, mods};

#[derive(Debug, Clone)]
pub enum Event {
    Client(client::Event),
    Caps(mods::caps::Event),
    Chat(mods::chat::Event),
    Ping(mods::ping::Event),
    Profile(mods::profile::Event),
    Status(mods::status::Event),
}
