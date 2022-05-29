// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use std::sync::{Arc, RwLock};

use jid::{BareJid, JidParseError};
use libstrophe::{Connection, Context};

use super::ProseClientOrigin;
use crate::broker::ProseBroker;

// -- Structures --

pub struct ProseClientAccount {
    credentials: ProseClientAccountCredentials,
    states: ProseClientAccountStates,

    pub broker: Option<ProseBroker>,
}

#[derive(Default)]
struct ProseClientAccountStates {
    connected: bool,
}

#[derive(Default)]
pub struct ProseClientAccountBuilder {
    credentials: Option<ProseClientAccountCredentials>,
}

#[derive(Debug)]
pub enum ProseClientAccountBuilderError {
    CredentialsNotSet,
}

pub struct ProseClientAccountCredentials {
    pub jid: BareJid,
    pub password: String,
    pub origin: ProseClientOrigin,
}

#[derive(Debug)]
pub enum ProseClientAccountError {
    AlreadyConnected,
    AlreadyDisconnected,
    CannotConnect(JidParseError),
    InvalidCredentials,
    DoesNotExist,
    Unknown,
}

// -- Implementations --

impl ProseClientAccountBuilder {
    pub fn new() -> Self {
        return ProseClientAccountBuilder::default();
    }

    pub fn credentials(
        mut self,
        jid: BareJid,
        password: String,
        origin: ProseClientOrigin,
    ) -> Self {
        self.credentials = Some(ProseClientAccountCredentials {
            jid,
            password,
            origin,
        });

        self
    }

    pub fn build(self) -> Result<ProseClientAccount, ProseClientAccountBuilderError> {
        let credentials = self
            .credentials
            .ok_or(ProseClientAccountBuilderError::CredentialsNotSet)?;

        log::trace!("built prose client account with jid: {}", credentials.jid);

        Ok(ProseClientAccount {
            credentials,
            states: ProseClientAccountStates::default(),
            broker: None,
        })
    }
}

impl ProseClientAccount {
    pub fn connect(&mut self) -> Result<(), ProseClientAccountError> {
        let jid_string = self.credentials.jid.to_string();

        log::trace!("connect network for account jid: {}", &jid_string);

        // Already connected? Fail.
        if self.states.connected {
            return Err(ProseClientAccountError::AlreadyConnected);
        }

        // Mark as connected (right away)
        self.states.connected = true;

        // Create XMPP client
        log::trace!("create client for account jid: {}", &jid_string);

        let mut connection = Connection::new(Context::new_with_default_logger());

        // TODO

        // Assign XMPP client to broker
        // self.broker = Some(ProseBroker::new(Arc::new(RwLock::new(client))));

        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), ProseClientAccountError> {
        log::trace!(
            "disconnect network for account jid: {}",
            self.credentials.jid
        );

        // Already disconnected? Fail.
        if !self.states.connected {
            return Err(ProseClientAccountError::AlreadyDisconnected);
        }

        // Stop XMPP client stream
        // TODO

        // Stop broker thread
        // TODO

        Ok(())
    }

    pub fn broker<'a>(&'a self) -> Option<&'a ProseBroker> {
        log::trace!("acquire broker for account jid: {}", self.credentials.jid);

        self.broker.as_ref()
    }
}
