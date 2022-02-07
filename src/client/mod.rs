// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod account;

// -- Imports --

use std::collections::HashMap;
use std::str::FromStr;

use jid::BareJid;
use log;
use tokio_xmpp;
use xmpp_parsers;

use account::{ProseClientAccount, ProseClientAccountBuilder, ProseClientAccountBuilderError};

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

    pub fn add(&self, jid: &str, password: &str) -> Result<(), ProseClientAccountBuilderError> {
        log::trace!("got request to add account to prose client: {}", jid);

        // TODO: give back account reference, or JID?
        // TODO: return error if account already exists

        let jid_bare =
            BareJid::from_str(jid).or(Err(ProseClientAccountBuilderError::InvalidJID))?;

        let account = ProseClientAccountBuilder::new()
            .credentials(jid_bare, password.to_string(), self.origin)
            .build()?;

        // TODO: append account to hmap
        // TODO: store account creds in accounts DB

        if self.bound {
            log::trace!("will auto-connect account: {}", jid);

            // TODO: if already bound, auto-connect there
        }

        Ok(())
    }

    pub fn remove(&self, jid: &str) -> Result<(), ()> {
        log::trace!("got request to remove account from prose client: {}", jid);

        // TODO
        // TODO: pull from DB

        Err(())
    }

    pub fn update(&self, jid: &str, password: &str) -> Result<(), ()> {
        log::trace!("got request to update account in prose client: {}", jid);

        // TODO
        // TODO: update in DB

        Err(())
    }
}
