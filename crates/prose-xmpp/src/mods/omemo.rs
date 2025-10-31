// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::legacy_omemo::Bundle;
use xmpp_parsers::pubsub;
use xmpp_parsers::pubsub::pubsub::PublishOptions;
use xmpp_parsers::pubsub::{ItemId, NodeName};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::stanza::omemo::DeviceList;
use crate::util::{ItemIdExt, PubSubItemsExt, PubSubQuery};
use crate::{ns, PublishOptionsExt, RequestError};

/// XEP-0384: OMEMO Encryption
/// https://xmpp.org/extensions/xep-0384.html#usecases-building
#[derive(Default, Clone)]
pub struct OMEMO {
    ctx: ModuleContext,
}

impl Module for OMEMO {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl OMEMO {
    pub async fn load_device_list(&self, from: &BareJid) -> Result<DeviceList> {
        let device_list = self
            .ctx
            .query_pubsub_node(
                PubSubQuery::new(self.ctx.generate_id(), ns::LEGACY_OMEMO_DEVICELIST)
                    .set_to(from.clone()),
            )
            .await?
            .unwrap_or_default()
            .find_first_payload::<DeviceList>("list", ns::LEGACY_OMEMO)?
            .unwrap_or_default();

        Ok(device_list)
    }

    pub async fn publish_device_list(&self, device_list: DeviceList) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::LEGACY_OMEMO_DEVICELIST.to_string()),
                    items: vec![pubsub::pubsub::Item {
                        id: Some(ItemId::current()),
                        publisher: None,
                        payload: Some(device_list.into()),
                    }],
                },
                publish_options: Some(PublishOptions::for_public_data(None)),
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn delete_device_list(&self) -> Result<(), RequestError> {
        match self
            .ctx
            .delete_pubsub_node(ns::LEGACY_OMEMO_DEVICELIST)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) if err.is_item_not_found_err() => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn load_device_bundle(
        &self,
        from: &BareJid,
        device_id: u32,
    ) -> Result<Option<Bundle>> {
        let Some(items) = self
            .ctx
            .query_pubsub_node(
                PubSubQuery::new(
                    self.ctx.generate_id(),
                    format!("{}:{device_id}", ns::LEGACY_OMEMO_BUNDLES),
                )
                .set_to(from.clone()),
            )
            .await?
        else {
            return Ok(None);
        };

        Ok(items.find_first_payload::<Bundle>("bundle", ns::LEGACY_OMEMO)?)
    }

    pub async fn publish_device_bundle(&self, device_id: u32, bundle: Bundle) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(format!("{}:{device_id}", ns::LEGACY_OMEMO_BUNDLES)),
                    items: vec![pubsub::pubsub::Item {
                        id: Some(ItemId::current()),
                        publisher: None,
                        payload: Some(bundle.into()),
                    }],
                },
                publish_options: Some(PublishOptions::for_public_data(None)),
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn delete_device_bundle(&self, device_id: u32) -> Result<(), RequestError> {
        match self
            .ctx
            .delete_pubsub_node(format!("{}:{device_id}", ns::LEGACY_OMEMO_BUNDLES))
            .await
        {
            Ok(_) => Ok(()),
            Err(err) if err.is_item_not_found_err() => Ok(()),
            Err(err) => Err(err),
        }
    }
}
