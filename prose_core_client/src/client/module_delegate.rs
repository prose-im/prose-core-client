use std::sync::Arc;

use chrono::Utc;
use jid::{BareJid, Jid};
use once_cell::sync::Lazy;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, warn};

use prose_core_lib::modules::caps::DiscoveryInfo;
use prose_core_lib::modules::profile::avatar::Metadata;
use prose_core_lib::modules::profile::VCard;
use prose_core_lib::modules::{CapsDelegate, ChatDelegate, ProfileDelegate, ReceivedMessage};
use prose_core_lib::stanza::message::chat_marker;
use prose_core_lib::stanza::presence::Caps;
use prose_core_lib::stanza::{Message, Presence};

use crate::cache::{AvatarCache, DataCache};
use crate::client::ClientContext;
use crate::domain_ext::UserProfile;
use crate::types::message_like::TimestampedMessage;
use crate::types::MessageLike;
use crate::ClientEvent;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to start Tokio runtime")
});

pub(crate) struct ModuleDelegate<D: DataCache + 'static, A: AvatarCache + 'static> {
    ctx: Arc<ClientContext<D, A>>,
}

impl<D: DataCache, A: AvatarCache> ModuleDelegate<D, A> {
    pub fn new(ctx: Arc<ClientContext<D, A>>) -> Self {
        ModuleDelegate { ctx }
    }
}

impl<D: DataCache, A: AvatarCache> CapsDelegate for ModuleDelegate<D, A> {
    fn handle_disco_request(&self, _node: &str) -> anyhow::Result<DiscoveryInfo<'static>> {
        // TODO: Handle the case where node doesn't match our node and return an error
        Ok((&self.ctx.capabilities).into())
    }

    fn handle_caps_presence(&self, from: &Jid, caps: Caps) {
        debug!("Received caps presence from {} {}", from, caps.to_string());
        //<presence to="marc@prose.org/bot_example" from="marc@prose.org/macOS"><c hash="sha-1" xmlns="http://jabber.org/protocol/caps" ver="9BPoRqqiLZL8XvdjqJljbuHoJLQ=" node="https://www.prose.org"/><x xmlns="vcard-temp:x:update"><photo>c3beb0150525c2485cde2c0a830481ca7a9fb3e5</photo></x></presence>
        // TODO: Handle caps presence
    }
}

impl<D: DataCache, A: AvatarCache> ProfileDelegate for ModuleDelegate<D, A> {
    fn vcard_did_change(&self, from: &Jid, vcard: &VCard) {
        debug!("New vcard for {} {}", from, vcard.to_string());

        let Some(profile): Option<UserProfile> = vcard.try_into().ok() else {
            return;
        };

        let ctx = self.ctx.clone();
        let from = BareJid::from(from.clone());

        RUNTIME.spawn(async move {
            match ctx.data_cache.insert_user_profile(&from, &profile) {
                Ok(_) => {
                    debug!("Finished saving user profile");
                    ctx.send_event(ClientEvent::ContactChanged { jid: from });
                }
                Err(err) => debug!("Failed to save user profile. {}", err),
            }
        });
    }

    fn avatar_metadata_did_change(&self, from: &Jid, metadata: &Metadata) {
        debug!("New metadata for {} {}", from, metadata.to_string());

        let Some(info) = metadata.info() else {
            return;
        };

        let Some(metadata) = info.try_into().ok() else {
            warn!("Received invalid metadata: {}", metadata);
            return;
        };

        let ctx = self.ctx.clone();
        let from = BareJid::from(from.clone());

        RUNTIME.spawn(async move {
            match ctx.data_cache.insert_avatar_metadata(&from, &metadata) {
                Ok(_) => (),
                Err(err) => error!("Failed to cache avatar metadata {}", err),
            }

            match ctx.load_and_cache_avatar_image(&from, &metadata).await {
                Ok(path) => {
                    debug!("Finished downloading and caching image to {:?}", path);
                    ctx.send_event(ClientEvent::ContactChanged { jid: from });
                }
                Err(err) => error!("Failed downloading and caching image. {}", err),
            }
        });
    }

    fn presence_did_change(&self, from: &Jid, presence: &Presence) {
        let jid = BareJid::from(from.clone());

        match self.ctx.data_cache.insert_presence(
            &jid,
            presence.kind(),
            presence.show(),
            presence.status(),
        ) {
            Ok(_) => (),
            Err(err) => error!("Failed to insert presence. {}", err),
        }
        self.ctx.send_event(ClientEvent::ContactChanged { jid })
    }
}

impl<D: DataCache, A: AvatarCache> ChatDelegate for ModuleDelegate<D, A> {
    fn did_receive_message(&self, message: ReceivedMessage) {
        let message_is_carbon = message.is_carbon();
        // TODO: Inject date from outside for testing
        let timestamped_message = TimestampedMessage {
            message,
            timestamp: Utc::now(),
        };
        let message = match MessageLike::try_from(timestamped_message) {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to parse received message: {}", err);
                return;
            }
        };

        let ctx = self.ctx.clone();

        RUNTIME.spawn(async move {
            debug!("Caching received message…");
            match ctx.data_cache.insert_messages([&message]) {
                Err(err) => error!("Failed to cache received message {}", err),
                Ok(_) => (),
            };

            let conversation = if message_is_carbon {
                &message.to
            } else {
                &message.from
            };
            ctx.send_event_for_message(conversation, &message);

            // Don't send delivery receipts for carbons or anything other than a regular message.
            if message_is_carbon || !message.payload.is_message() {
                return;
            }

            let Some(xmpp) = &(*ctx.xmpp.read().await) else {
                return;
            };

            match xmpp.chat.mark_message(
                &xmpp.client.context(),
                &message.id,
                message.from,
                chat_marker::Kind::Received,
            ) {
                Err(err) => error!("Failed to send delivery receipt {}", err),
                Ok(_) => debug!("Sent delivery receipt for message {}", message.id),
            }
        });
    }

    fn will_send_message(&self, message: &Message) {
        // TODO: Inject date from outside for testing
        let timestamped_message = TimestampedMessage {
            message,
            timestamp: Utc::now(),
        };

        let message = match MessageLike::try_from(timestamped_message) {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to parse received message: {}", err);
                return;
            }
        };

        let ctx = self.ctx.clone();

        RUNTIME.spawn(async move {
            debug!("Caching sent message…");
            match ctx.data_cache.insert_messages([&message]) {
                Err(err) => error!("Failed to cache received message {}", err),
                Ok(_) => (),
            };
            ctx.send_event_for_message(&message.to, &message);
        });
    }
}
