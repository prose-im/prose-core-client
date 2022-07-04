// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::error::Result;
use crate::account::{Account, AccountObserver};
use crate::connection::LibstropheConnection;
use crate::{ChatState, ShowKind};

use jid::BareJid;
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

    pub fn connect(&self, password: &str, observer: Box<dyn AccountObserver>) -> Result<()> {
        let connection = LibstropheConnection::new(&self.jid, password);
        let account = Account::new(Box::new(connection), observer)?;
        ACCOUNTS.lock()?.insert(self.jid.clone(), account);
        Ok(())
    }

    pub fn sendMessage(
        &self,
        id: &str,
        to: &BareJid,
        body: &str,
        chat_state: Option<ChatState>,
    ) -> Result<()> {
        with_account(&self.jid, |account| {
            account.chat.send_message(id, to, body, chat_state)
        })
    }

    pub fn updateMessage(&self, id: &str, new_id: &str, to: &BareJid, body: &str) -> Result<()> {
        with_account(&self.jid, |account| {
            account.chat.update_message(id, new_id, to, body)
        })
    }

    pub fn sendChatState(&self, to: &BareJid, chat_state: ChatState) -> Result<()> {
        with_account(&self.jid, |account| {
            account.chat.send_chat_state(to, chat_state)
        })
    }

    pub fn sendPresence(&self, show: Option<ShowKind>, status: &Option<String>) -> Result<()> {
        with_account(&self.jid, |account| {
            account.presence.send_presence(show, status.as_deref())
        })
    }

    pub fn loadRoster(&self) -> Result<()> {
        with_account(&self.jid, |account| account.roster.load_roster())
    }

    pub fn sendXMLPayload(&self, xml_str: &str) -> Result<()> {
        with_account(&self.jid, |account| account.debug.send_xml_payload(xml_str))
    }
}

fn with_account<T>(
    account_jid: &BareJid,
    handler: impl FnOnce(&mut Account) -> Result<T>,
) -> Result<T> {
    let mut locked_hash_map = ACCOUNTS.lock()?;
    let account = locked_hash_map
        .get_mut(&account_jid)
        .expect("Cannot get account. Did you call Client.connect()?");
    handler(account)
}
