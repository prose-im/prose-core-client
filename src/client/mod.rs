// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod account;

// -- Imports --

use std::collections::hash_map::Iter as HashMapIter;
use std::collections::HashMap;
use std::str::FromStr;

use jid::BareJid;
use log;

use account::{
    ProseClientAccount, ProseClientAccountBuilder, ProseClientAccountBuilderError,
    ProseClientAccountError,
};

// -- Structures --

#[derive(Clone, Copy, Debug)]
pub enum ProseClientOrigin {
    TestsCLI,
    ProseAppMacOS,
    ProseAppIOS,
    ProseAppAndroid,
    ProseAppWindows,
    ProseAppLinux,
    ProseAppWeb,
}

pub struct ProseClient {
    bound: bool,

    pub accounts: HashMap<BareJid, ProseClientAccount>,
    pub origin: ProseClientOrigin,
}

#[derive(Debug)]
pub enum ProseClientError {
    InvalidJID,
    AccountAlreadyExists,
    AccountNotFound,
    AccountError(ProseClientAccountError),
    AccountBuilderError(ProseClientAccountBuilderError),
    Unknown,
}

#[derive(Default)]
pub struct ProseClientBuilder {
    pub origin: Option<ProseClientOrigin>,
}

#[derive(Debug)]
pub enum ProseClientBuilderError {
    OriginNotSet,
}

#[derive(Debug)]
pub enum ProseClientBindError {
    AlreadyBound,
    Unknown,
}

pub enum ProseClientUnbindReason {
    Bye,
}

#[derive(Debug)]
pub enum ProseClientUnbindError {
    NotBound,
    Unknown,
}

// -- Implementations --

impl ProseClientBuilder {
    pub fn new() -> Self {
        return ProseClientBuilder::default();
    }

    pub fn app(mut self, origin: ProseClientOrigin) -> Self {
        self.origin = Some(origin);

        self
    }

    pub fn build(self) -> Result<ProseClient, ProseClientBuilderError> {
        let origin = self.origin.ok_or(ProseClientBuilderError::OriginNotSet)?;

        log::trace!("built prose client with origin: {:?}", origin);

        Ok(ProseClient {
            bound: false,
            accounts: HashMap::new(),
            origin: origin,
        })
    }
}

impl ProseClient {
    // -- Client lifecycle --

    pub fn bind(mut self) -> Result<Self, ProseClientBindError> {
        if !self.bound {
            // Mark as bound immediately, so that added accounts auto-connect straight away
            self.bound = true;

            // TODO

            // TODO: load accounts from DB
            // TODO: build each account and bind them

            // TODO: connect each account

            // Err(ProseClientBindError::Unknown)

            log::trace!("bound prose client");

            Ok(self)
        } else {
            log::warn!("cannot bind prose client, because it is already bound");

            Err(ProseClientBindError::AlreadyBound)
        }
    }

    pub fn unbind(mut self, reason: ProseClientUnbindReason) -> Result<(), ProseClientUnbindError> {
        if self.bound {
            self.bound = false;

            // TODO: follow reverse process than done during bind()

            // TODO: disconnect each account
            // TODO: unload accounts from db

            log::trace!("unbound prose client");

            Err(ProseClientUnbindError::Unknown)
        } else {
            log::warn!("cannot unbind prose client, because it is not bound");

            Err(ProseClientUnbindError::NotBound)
        }
    }

    // -- Account management --

    pub fn add(&mut self, jid: &str, password: &str) -> Result<BareJid, ProseClientError> {
        log::trace!("got request to add account to prose client: {}", jid);

        let jid_bare = BareJid::from_str(jid).or(Err(ProseClientError::InvalidJID))?;

        // Account already exists locally? (cannot add twice)
        if self.accounts.contains_key(&jid_bare) {
            return Err(ProseClientError::AccountAlreadyExists);
        }

        // Build account and insert in global map of accounts
        let account = ProseClientAccountBuilder::new()
            .credentials(jid_bare.clone(), password.to_string(), self.origin)
            .build()
            .map_err(|err| ProseClientError::AccountBuilderError(err))?;

        self.accounts.insert(jid_bare.clone(), account);

        // Store account in database
        // TODO

        if self.bound {
            log::trace!("will auto-connect account: {} (after add)", jid);

            // Acquire account from local store
            let mut account = self
                .accounts
                .get_mut(&jid_bare)
                .ok_or(ProseClientError::AccountNotFound)?;

            // Connect to account
            // Notice: now that the account was added, any connection error are sent over the \
            //   event broker. This is asynchronous, so the library user needs to bind itself to \
            //   ingress events.
            account
                .connect()
                .map_err(|err| ProseClientError::AccountError(err))?;
        }

        Ok(jid_bare)
    }

    pub fn remove(&self, jid: &str) -> Result<(), ()> {
        log::trace!("got request to remove account from prose client: {}", jid);

        if self.bound {
            log::trace!("will auto-disconnect account: {} (after remove)", jid);

            // TODO: if already bound, auto-disconnect there
        }

        // TODO: pull from DB
        // TODO: disconnect if bound (call .disconnect())

        Err(())
    }

    pub fn get<'a>(&'a self, jid: &str) -> Result<&'a ProseClientAccount, ProseClientError> {
        log::trace!("got request to get account from prose client: {}", jid);

        let jid_bare = BareJid::from_str(jid).or(Err(ProseClientError::InvalidJID))?;

        // Acquire account from local accounts store
        self.accounts
            .get(&jid_bare)
            .ok_or(ProseClientError::AccountNotFound)
    }

    pub fn iter(&self) -> HashMapIter<BareJid, ProseClientAccount> {
        log::trace!("got request to iterate on all accounts from prose client");

        self.accounts.iter()
    }
}
