// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::Roster;

use super::types::message::Message;

#[allow(non_snake_case)]
pub trait AccountObserver: Send + Sync {
    fn didConnect(&self);
    fn didDisconnect(&self);

    fn didReceiveMessage(&self, message: Message);
    fn didReceiveRoster(&self, roster: Roster);
}
