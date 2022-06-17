use super::account::Account;
use super::account_observer::AccountObserver;
use super::LoginError;
use jid::BareJid;
use std::{str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

static ACCOUNTS: Lazy<Mutex<HashMap<BareJid, Account>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Client {}

#[allow(non_snake_case)]
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

    pub fn sendMessage(&self, account_jid_str: &str, receiver_jid_str: &str, body: &str) {
        let receiver_jid = BareJid::from_str(receiver_jid_str).expect("Cannot parse receiver JID");
        with_account(account_jid_str, |account| {
            account.send_message(&receiver_jid, body);
        });
    }

    pub fn loadRoster(&self, account_jid_str: &str) {
        with_account(account_jid_str, |account| {
            account.load_roster();
        });
    }
}

fn with_account<T, F>(account_jid_str: &str, handler: F) -> T
where
    F: FnOnce(&Account) -> T,
{
    let account_jid = BareJid::from_str(account_jid_str).expect("Cannot parse account JID");
    let locked_hash_map = ACCOUNTS.lock().unwrap();
    let account = locked_hash_map
        .get(&account_jid)
        .expect("Cannot get account");
    handler(&account)
}
