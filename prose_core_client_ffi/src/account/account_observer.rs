// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{Presence, Roster};

use jid::BareJid;
#[cfg(feature = "test-helpers")]
use std::sync::{Arc, Mutex};

use crate::Message;

#[cfg_attr(feature = "test-helpers", mockiato::mockable)]
pub trait AccountObserver: Send + Sync {
    fn did_connect(&self);
    fn did_disconnect(&self);

    fn did_receive_message(&self, message: Message);
    fn did_receive_roster(&self, roster: Roster);
    fn did_receive_presence(&self, presence: Presence);
    fn did_receive_presence_subscription_request(&self, from: BareJid);
}

#[cfg(feature = "test-helpers")]
impl<'mock> AccountObserver for Arc<Mutex<AccountObserverMock<'mock>>> {
    fn did_connect(&self) {
        self.lock().unwrap().did_connect();
    }
    fn did_disconnect(&self) {
        self.lock().unwrap().did_disconnect();
    }

    fn did_receive_message(&self, message: Message) {
        self.lock().unwrap().did_receive_message(message);
    }
    fn did_receive_roster(&self, roster: Roster) {
        self.lock().unwrap().did_receive_roster(roster);
    }
    fn did_receive_presence(&self, presence: Presence) {
        self.lock().unwrap().did_receive_presence(presence);
    }
    fn did_receive_presence_subscription_request(&self, from: BareJid) {
        self.lock()
            .unwrap()
            .did_receive_presence_subscription_request(from);
    }
}

#[cfg(feature = "test-helpers")]
unsafe impl<'mock> Send for AccountObserverMock<'mock> {}
#[cfg(feature = "test-helpers")]
unsafe impl<'mock> Sync for AccountObserverMock<'mock> {}
