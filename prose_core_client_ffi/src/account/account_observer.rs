// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{Presence, Roster};

use crate::Message;

pub trait AccountObserver: Send + Sync {
    fn did_connect(&self);
    fn did_disconnect(&self);

    fn did_receive_message(&self, message: Message);
    fn did_receive_roster(&self, roster: Roster);
    fn did_receive_presence(&self, presence: Presence);
}
