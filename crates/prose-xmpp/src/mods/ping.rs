use anyhow::Result;
use tracing::info;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::ping;

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::util::RequestError;

#[derive(Default, Clone)]
pub(crate) struct Ping {
    ctx: ModuleContext,
}

impl Module for Ping {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl Ping {
    pub async fn send_ping(&self) -> Result<()> {
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
