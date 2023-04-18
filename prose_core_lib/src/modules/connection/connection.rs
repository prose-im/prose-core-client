use std::time::Duration;
use tracing::info;

use crate::modules::{Context, Module, RequestError};
use crate::stanza::iq::Kind::Get;
use crate::stanza::{Namespace, Presence, Stanza, StanzaBase, IQ};

pub(crate) struct Connection {}

impl Connection {
    pub(crate) fn new() -> Self {
        Connection {}
    }
}

impl Module for Connection {}

impl Connection {
    // After establishing a session, a client SHOULD send initial presence to
    // the server in order to signal its availability for communications. As
    // defined herein, the initial presence stanza (1) MUST possess no 'to'
    // address (signalling that it is meant to be broadcast by the server on
    // behalf of the client) and (2) MUST possess no 'type' attribute
    // (signalling the user's availability). After sending initial presence,
    // an active resource is said to be an "available resource".
    pub fn send_initial_presence(&self, ctx: &Context) -> anyhow::Result<()> {
        ctx.send_stanza(Presence::new());
        Ok(())
    }

    pub fn send_ping(&self, ctx: &Context) -> anyhow::Result<()> {
        let iq = IQ::new(Get, ctx.generate_id())
            .set_from(ctx.jid.clone())
            .add_child(Stanza::new("ping").set_namespace(Namespace::Ping));

        ctx.send_iq_with_timeout_cb(iq, Duration::from_secs(5), |ctx, result| {
            if let Err(RequestError::TimedOut) = result {
                info!("Ping timed out. Disconnecting…");
                ctx.disconnect();
            }
        })
    }
}
