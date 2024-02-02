// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::blocking::{Block, BlocklistRequest, BlocklistResult, Unblock};
use xmpp_parsers::iq::{Iq, IqType};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::{ns, RequestError};

#[derive(Default, Clone)]
pub struct BlockList {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    UserBlocked { jid: Jid },
    UserUnblocked { jid: Jid },
    BlockListCleared,
}

impl Module for BlockList {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_iq_stanza(&self, stanza: &Iq) -> Result<()> {
        let IqType::Set(payload) = &stanza.payload else {
            return Ok(());
        };

        let event = match payload {
            _ if payload.is("block", ns::BLOCKING) => {
                let mut block = Block::try_from(payload.clone())?;
                let Some(jid) = block.items.pop() else {
                    return Ok(());
                };
                Event::UserBlocked { jid }
            }
            _ if payload.is("unblock", ns::BLOCKING) => {
                let mut unblock = Unblock::try_from(payload.clone())?;
                if let Some(jid) = unblock.items.pop() {
                    Event::UserUnblocked { jid }
                } else {
                    Event::BlockListCleared
                }
            }
            _ => return Ok(()),
        };

        self.ctx.schedule_event(ClientEvent::BlockList(event));
        Ok(())
    }
}

/// https://xmpp.org/extensions/xep-0191.html
impl BlockList {
    pub async fn load_block_list(&self) -> Result<Vec<Jid>> {
        let response = self
            .ctx
            .send_iq(Iq::from_get(self.ctx.generate_id(), BlocklistRequest {}))
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        let block_list = BlocklistResult::try_from(response)?;
        Ok(block_list.items)
    }

    /// https://xmpp.org/extensions/xep-0191.html#block
    pub async fn block_user(&self, jid: &Jid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Block {
                items: vec![jid.clone()],
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    /// https://xmpp.org/extensions/xep-0191.html#unblock
    pub async fn unblock_user(&self, jid: &Jid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Unblock {
                items: vec![jid.clone()],
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    /// https://xmpp.org/extensions/xep-0191.html#unblockall
    pub async fn clear_block_list(&self) -> Result<()> {
        let iq = Iq::from_set(self.ctx.generate_id(), Unblock { items: vec![] });
        self.ctx.send_iq(iq).await?;
        Ok(())
    }
}
