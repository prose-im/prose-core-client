// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::any::Any;

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::pubsub::PubSubEvent;

pub use block_list::BlockList;
pub use bookmark::Bookmark;
pub use bookmark2::Bookmark as Bookmark2;
pub use caps::Caps;
pub use chat::Chat;
pub use http_upload::HttpUpload;
pub use mam::MAM;
pub use muc::MUC;
pub use omemo::OMEMO;
pub use ping::Ping;
pub use profile::{AvatarData, Profile};
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
pub use pubsub::PubSub;
pub use roster::Roster;
pub use status::Status;

use crate::client::ModuleContext;
use crate::stanza::{Message, PubSubMessage};
use crate::util::XMPPElement;

pub mod block_list;
pub mod bookmark;
pub mod bookmark2;
pub mod caps;
pub mod chat;
pub mod http_upload;
pub mod mam;
pub mod muc;
pub mod omemo;
pub mod ping;
pub mod profile;
pub mod pubsub;
pub mod roster;
pub mod status;

pub trait Module: Any + SendUnlessWasm + SyncUnlessWasm {
    fn register_with(&mut self, context: ModuleContext);

    fn handle_connect(&self) -> Result<()> {
        Ok(())
    }

    fn handle_element(&self, element: &XMPPElement) -> Result<()> {
        match element {
            XMPPElement::Presence(ref p) => self.handle_presence_stanza(p),
            XMPPElement::Message(ref m) => self.handle_message_stanza(m),
            XMPPElement::IQ(ref i) => self.handle_iq_stanza(i),
            XMPPElement::PubSubMessage(ref m) => self.handle_pubsub_message(m),
        }
    }

    fn handle_pubsub_message(&self, pubsub: &PubSubMessage) -> Result<()> {
        for event in pubsub.events.iter() {
            self.handle_pubsub_event(&pubsub.from, event)?
        }
        Ok(())
    }

    fn handle_presence_stanza(&self, _stanza: &Presence) -> Result<()> {
        Ok(())
    }
    fn handle_message_stanza(&self, _stanza: &Message) -> Result<()> {
        Ok(())
    }
    fn handle_iq_stanza(&self, _stanza: &Iq) -> Result<()> {
        Ok(())
    }
    fn handle_pubsub_event(&self, _from: &Jid, _event: &PubSubEvent) -> Result<()> {
        Ok(())
    }
}

pub trait AnyModule: Module {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Module> AnyModule for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
