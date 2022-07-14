// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::error::Result;
use crate::account::{Account, AccountObserver, UUIDProvider};
use crate::connection::LibstropheConnection;
use crate::types::message::ChatState;
use crate::types::presence::ShowKind;
use crate::XMPPMAMPreferences;
use jid::BareJid;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

static ACCOUNTS: Lazy<Mutex<HashMap<BareJid, Account>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Client {
    jid: BareJid,
}

impl Client {
    pub fn new(jid: BareJid) -> Self {
        Client { jid }
    }

    pub fn jid(&self) -> BareJid {
        self.jid.clone()
    }

    pub fn connect(&self, password: &str, observer: Box<dyn AccountObserver>) -> Result<()> {
        let connection = LibstropheConnection::new(&self.jid, password);
        let account = Account::new(
            Box::new(connection),
            Box::new(UUIDProvider::new()),
            observer,
        )?;
        ACCOUNTS.lock()?.insert(self.jid.clone(), account);
        Ok(())
    }

    pub fn send_message(
        &self,
        id: &str,
        to: &BareJid,
        body: &str,
        chat_state: Option<ChatState>,
    ) -> Result<()> {
        self.with_account(|account| account.chat.send_message(id, to, body, chat_state))
    }

    pub fn update_message(&self, id: &str, new_id: &str, to: &BareJid, body: &str) -> Result<()> {
        self.with_account(|account| account.chat.update_message(id, new_id, to, body))
    }

    pub fn send_chat_state(&self, to: &BareJid, chat_state: ChatState) -> Result<()> {
        self.with_account(|account| account.chat.send_chat_state(to, chat_state))
    }

    pub fn send_presence(&self, show: Option<ShowKind>, status: &Option<String>) -> Result<()> {
        self.with_account(|account| account.presence.send_presence(show, status.as_deref()))
    }

    pub fn load_roster(&self) -> Result<()> {
        self.with_account(|account| account.roster.load_roster())
    }

    pub fn add_user(
        &self,
        jid: &BareJid,
        nickname: &Option<String>,
        groups: &Vec<String>,
    ) -> Result<()> {
        self.with_account(|account| account.roster.add_user(jid, nickname.as_deref(), groups))
    }

    pub fn remove_user_and_unsubscribe_from_presence(&self, jid: &BareJid) -> Result<()> {
        self.with_account(|account| {
            account
                .roster
                .remove_user_and_unsubscribe_from_presence(jid)
        })
    }

    pub fn subscribe_to_user_presence(&self, jid: &BareJid) -> Result<()> {
        self.with_account(|account| account.roster.subscribe_to_user_presence(jid))
    }

    pub fn unsubscribe_from_user_presence(&self, jid: &BareJid) -> Result<()> {
        self.with_account(|account| account.roster.unsubscribe_from_user_presence(jid))
    }

    pub fn grant_presence_permission_to_user(&self, jid: &BareJid) -> Result<()> {
        self.with_account(|account| account.roster.grant_presence_permission_to_user(jid))
    }

    pub fn revoke_or_reject_presence_permission_from_user(&self, jid: &BareJid) -> Result<()> {
        self.with_account(|account| {
            account
                .roster
                .revoke_or_reject_presence_permission_from_user(jid)
        })
    }

    pub fn load_archiving_preferences(&self) -> Result<()> {
        self.with_account(|account| account.mam.load_archiving_preferences())
    }

    pub fn set_archiving_preferences(&self, preferences: &XMPPMAMPreferences) -> Result<()> {
        self.with_account(|account| account.mam.set_archiving_preferences(preferences))
    }

    pub fn load_messages_in_chat(
        &self,
        request_id: &str,
        jid: &BareJid,
        before: &Option<String>,
    ) -> Result<()> {
        self.with_account(|account| {
            account
                .mam
                .load_messages_in_chat(request_id, jid, before.as_deref())
        })
    }

    pub fn send_xml_payload(&self, xml_str: &str) -> Result<()> {
        self.with_account(|account| account.debug.send_xml_payload(xml_str))
    }
}

impl Client {
    fn with_account<T>(&self, handler: impl FnOnce(&mut Account) -> Result<T>) -> Result<T> {
        let mut locked_hash_map = ACCOUNTS.lock()?;
        let account = locked_hash_map
            .get_mut(&self.jid)
            .expect("Cannot get account. Did you call Client.connect()?");
        handler(account)
    }
}
