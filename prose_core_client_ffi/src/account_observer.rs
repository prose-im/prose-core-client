use crate::Roster;

use super::types::message::Message;

#[allow(non_snake_case)]
pub trait AccountObserver: Send + Sync {
    fn didConnect(&self);
    fn didDisconnect(&self);

    fn didReceiveMessage(&self, message: Message);
    fn didReceiveRoster(&self, roster: Roster);
}
