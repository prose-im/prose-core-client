// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod account;

use jid::BareJid;
use std::{str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

use account::Account;

pub trait AccountObserver: Send + Sync {
    fn didConnect(&self);
    fn didDisconnect(&self);

    fn didReceive(&self, message: Message);
}

static ACCOUNTS: Lazy<Mutex<HashMap<BareJid, Account>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Message {
    from: BareJid,
    body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("JID is invalid")]
    InvalidJID,
}

struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub fn connect(
        &self,
        jid_str: &str,
        password: &str,
        observer: Box<dyn AccountObserver>,
    ) -> Result<BareJid, LoginError> {
        let jid = BareJid::from_str(jid_str).or(Err(LoginError::InvalidJID))?;

        let account = Account::new(&jid, password, Arc::new(observer));
        ACCOUNTS.lock().unwrap().insert(jid.clone(), account);

        Ok(jid)
    }

    pub fn sendMessage(&self, jid_str: &str, body: &str) {
        let jid = BareJid::from_str(jid_str).expect("Cannot parse JID");
        let locked_hash_map = ACCOUNTS.lock().unwrap();
        let account = locked_hash_map.get(&jid).expect("Cannot get account");
        account.send_message(&jid, body);
    }
}

uniffi_macros::include_scaffolding!("ProseCoreClientFFI");
