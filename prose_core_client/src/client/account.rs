// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use core::time::Duration;

use jid::{BareJid, JidParseError};
use libstrophe::{Connection, ConnectionEvent, ConnectionFlags, Context};

use super::{event::ProseClientEvent, ProseClientOrigin};
use crate::broker::ProseBroker;

// -- Constants --

const CLIENT_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(180);
const CLIENT_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(60);

// -- Structures --

pub struct ProseClientAccount<'cl, 'cb> {
    credentials: ProseClientAccountCredentials,
    states: ProseClientAccountStates,

    pub broker: Option<ProseBroker<'cl, 'cb, 'static>>,
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

    pub fn build<'cl, 'cb>(
        self,
    ) -> Result<ProseClientAccount<'cl, 'cb>, ProseClientAccountBuilderError> {
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

impl<'cl, 'cb> ProseClientAccount<'cl, 'cb> {
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

        connection
            .set_flags(ConnectionFlags::MANDATORY_TLS)
            .or(Err(ProseClientAccountError::Unknown))?;
        connection.set_keepalive(CLIENT_KEEPALIVE_TIMEOUT, CLIENT_KEEPALIVE_INTERVAL);

        connection.set_jid(jid_string);
        connection.set_pass(&self.credentials.password);

        // Connect XMPP client
        // TODO: move this to a thread (go async)
        let context = connection
            .connect_client(
                None,
                None,
                |context: &Context, connection: &mut Connection, event: ConnectionEvent| {
                    // TODO: handle connect event there, and register \
                    //   reference to connection in broker from there. If \
                    //   disconnected, un-register broker.

                    ProseClientEvent::connection(context, connection, event)
                },
            )
            .expect("cannot connect to server");

        context.run();

        // Assign XMPP client to broker
        // TODO
        //         let client = ProseBrokerClient::from_connection(connection);
        //         let broker = ProseBroker::from_client(&client);
        //
        //         self.broker = Some(broker);

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

    pub fn broker<'a>(&'a self) -> Option<&'a ProseBroker<'cl, 'cb, 'static>> {
        log::trace!("acquire broker for account jid: {}", self.credentials.jid);

        self.broker.as_ref()
    }
}
