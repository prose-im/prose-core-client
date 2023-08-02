use crate::{client, mods};

#[derive(Debug, Clone)]
pub enum Event {
    Client(client::Event),
    Caps(mods::caps::Event),
    Chat(mods::chat::Event),
    Profile(mods::profile::Event),
    Status(mods::status::Event),
}
