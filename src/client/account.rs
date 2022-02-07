// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use jid::BareJid;

use super::ProseClientOrigin;
use crate::broker::ProseBroker;

pub struct ProseClientAccount {
    credentials: ProseClientAccountCredentials,

    pub broker: ProseBroker,
}

#[derive(Default)]
pub struct ProseClientAccountBuilder {
    credentials: Option<ProseClientAccountCredentials>,
}

#[derive(Debug)]
pub enum ProseClientAccountBuilderError {
    InvalidJID,
    CredentialsNotSet,
}

pub struct ProseClientAccountCredentials {
    pub jid: BareJid,
    pub password: String,
    pub origin: ProseClientOrigin,
}

pub enum ProseClientAccountError {
    InvalidCredentials,
    DoesNotExist,
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
            jid: jid,
            password: password,
            origin: origin,
        });

        self
    }

    pub fn build(self) -> Result<ProseClientAccount, ProseClientAccountBuilderError> {
        let credentials = self
            .credentials
            .ok_or(ProseClientAccountBuilderError::CredentialsNotSet)?;

        log::trace!("built prose client account with jid: {}", credentials.jid);

        Ok(ProseClientAccount {
            credentials: credentials,
            broker: ProseBroker::new(),
        })
    }
}
