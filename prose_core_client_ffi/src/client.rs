// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{ChatState, ConnectionError, ShowKind};

use super::account::Account;
use super::account_observer::AccountObserver;
use jid::BareJid;
use std::sync::Arc;

use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

static ACCOUNTS: Lazy<Mutex<HashMap<BareJid, Account>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Client {
    jid: BareJid,
}

#[allow(non_snake_case)]
impl Client {
    pub fn new(jid: BareJid) -> Self {
        Client { jid }
    }

    pub fn jid(&self) -> BareJid {
        self.jid.clone()
    }

    pub fn connect(
        &self,
        password: &str,
        observer: Box<dyn AccountObserver>,
    ) -> Result<(), ConnectionError> {
        let account = Account::new(&self.jid, password, Arc::new(observer))?;
        ACCOUNTS.lock().unwrap().insert(self.jid.clone(), account);
        Ok(())
    }

    pub fn sendMessage(&self, id: &str, to: &BareJid, body: &str, chat_state: Option<ChatState>) {
        with_account(&self.jid, |account| {
            account.send_message(id, to, body, chat_state);
        });
    }

    pub fn updateMessage(&self, id: &str, new_id: &str, to: &BareJid, body: &str) {
        with_account(&self.jid, |account| {
            account.update_message(id, new_id, to, body);
        });
    }

    pub fn sendChatState(&self, to: &BareJid, chat_state: ChatState) {
        with_account(&self.jid, |account| {
            account.send_chat_state(to, chat_state);
        });
    }

    pub fn sendPresence(&self, show: Option<ShowKind>, status: &Option<String>) {
        with_account(&self.jid, |account| {
            account.send_presence(show, status.as_deref());
        });
    }

    pub fn loadRoster(&self) {
        with_account(&self.jid, |account| {
            account.load_roster();
        });
    }

    pub fn sendXMLPayload(&self, xml_str: &str) {
        with_account(&self.jid, |account| {
            account.send_xml_payload(xml_str);
        });
    }
}

fn with_account<T, F>(account_jid: &BareJid, handler: F) -> T
where
    F: FnOnce(&Account) -> T,
{
    let locked_hash_map = ACCOUNTS.lock().unwrap();
    let account = locked_hash_map
        .get(&account_jid)
        .expect("Cannot get account");
    handler(&account)
}
