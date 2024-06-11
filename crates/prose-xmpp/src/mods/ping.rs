// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use tracing::info;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::ping;

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::ns;
use crate::util::RequestError;

// XEP-0199: XMPP Ping
// https://xmpp.org/extensions/xep-0199.html

#[derive(Default, Clone)]
pub struct Ping {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Ping { from: Jid, id: String },
}

impl Module for Ping {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_iq_stanza(&self, stanza: &Iq) -> Result<()> {
        let IqType::Get(payload) = &stanza.payload else {
            return Ok(());
        };

        if payload.is("ping", ns::PING) {
            let Some(from) = &stanza.from else {
                bail!("Missing 'from' in ping request.")
            };
            self.ctx.schedule_event(ClientEvent::Ping(Event::Ping {
                from: from.clone(),
                id: stanza.id.clone(),
            }))
        }

        Ok(())
    }
}

impl Ping {
    pub async fn send_ping(&self, to: impl Into<Jid>) -> Result<(), RequestError> {
        self.ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), ping::Ping)
                    .with_from(self.ctx.full_jid().clone().into())
                    .with_to(to.into()),
            )
            .await
            .map(|_| ())
    }

    pub async fn send_pong(&self, to: Jid, id: impl AsRef<str>) -> Result<(), RequestError> {
        let iq = Iq {
            from: None,
            to: Some(to),
            id: id.as_ref().to_string(),
            payload: IqType::Result(None),
        };
        self.ctx.send_stanza(iq)?;
        Ok(())
    }
}

impl Ping {
    pub(crate) async fn send_ping_to_server(&self) -> Result<()> {
        let result = self
            .ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), ping::Ping)
                    .with_from(self.ctx.full_jid().clone().into()),
            )
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(RequestError::TimedOut) => {
                info!("Ping timed out. Disconnecting…");
                self.ctx.disconnect();
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}
