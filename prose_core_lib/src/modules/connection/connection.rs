use std::time::Duration;

use tracing::info;

use crate::modules::{Context, Module, RequestError};
use crate::stanza::iq::Kind::Get;
use crate::stanza::{Namespace, Stanza, StanzaBase, IQ};

pub(crate) struct Connection {}

impl Connection {
    pub(crate) fn new() -> Self {
        Connection {}
    }
}

impl Module for Connection {}

impl Connection {
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
