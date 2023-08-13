// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::stanza::UserActivity;
use crate::{ns, Event as ClientEvent};
use anyhow::Result;
use jid::Jid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::pubsub::{NodeName, PubSub, PubSubEvent};
use xmpp_parsers::{presence, pubsub};

#[derive(Default, Clone)]
pub struct Status {
    ctx: ModuleContext,
}

#[derive(Debug, Clone)]
pub enum Event {
    Presence(Presence),
    UserActivity {
        from: Jid,
        user_activity: UserActivity,
    },
}

impl Module for Status {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context;
    }

    fn handle_presence_stanza(&self, stanza: &Presence) -> Result<()> {
        self.ctx
            .schedule_event(ClientEvent::Status(Event::Presence(stanza.clone())));
        Ok(())
    }

    fn handle_pubsub_event(&self, from: &Jid, event: &PubSubEvent) -> Result<()> {
        let PubSubEvent::PublishedItems { node, items } = event else {
            return Ok(());
        };

        match node.0.as_ref() {
            ns::USER_ACTIVITY => {
                let Some(item) = items.first() else {
                    return Ok(());
                };
                let Some(payload) = &item.payload else {
                    return Ok(());
                };
                let user_activity = UserActivity::try_from(payload.clone())?;
                self.ctx
                    .schedule_event(ClientEvent::Status(Event::UserActivity {
                        from: from.clone(),
                        user_activity,
                    }));
            }
            _ => (),
        }
        Ok(())
    }
}

impl Status {
    /// XMPP: Instant Messaging and Presence
    /// https://xmpp.org/rfcs/rfc6121.html#presence
    pub fn send_presence(&self, show: Option<presence::Show>, status: Option<&str>) -> Result<()> {
        let mut presence = Presence::new(presence::Type::None);
        presence.show = show;
        if let Some(status) = status {
            presence.set_status("", status);
        }
        self.ctx.send_stanza(presence)
    }

    /// XEP-0108: User Activity
    /// https://xmpp.org/extensions/xep-0108.html
    pub async fn publish_activity(&self, activity: UserActivity) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::USER_ACTIVITY.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: Some(pubsub::ItemId(self.ctx.bare_jid().to_string())),
                        publisher: None,
                        payload: Some(activity.into()),
                    })],
                },
                publish_options: None,
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }
}
