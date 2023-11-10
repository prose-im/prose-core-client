// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::disco::Item as DiscoItem;
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::pubsub::owner::Configure;
use xmpp_parsers::pubsub::pubsub::{Item, Items, PublishOptions, Retract};
use xmpp_parsers::pubsub::{pubsub, Item as PubSubItem, ItemId, NodeName};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::util::RequestError;
use crate::{ns, ElementExt};

#[derive(Default, Clone)]
pub struct PubSub {
    ctx: ModuleContext,
}

impl Module for PubSub {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl PubSub {
    pub async fn publish_items(
        &self,
        node: impl AsRef<str>,
        items: impl IntoIterator<Item = PubSubItem>,
        options: Option<PublishOptions>,
    ) -> Result<(), RequestError> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::Publish {
                    node: NodeName(node.as_ref().to_string()),
                    items: items.into_iter().map(|item| Item(item)).collect(),
                },
                publish_options: options,
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn load_items_list(
        &self,
        node: impl AsRef<str>,
    ) -> Result<Vec<DiscoItem>, RequestError> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            DiscoItemsQuery {
                node: Some(node.as_ref().to_string()),
            },
        );

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        Ok(DiscoItemsResult::try_from(response)?.items)
    }

    pub async fn load_items_with_ids<ID: AsRef<str>>(
        &self,
        node: impl AsRef<str>,
        item_ids: impl IntoIterator<Item = ID>,
    ) -> Result<Vec<PubSubItem>, RequestError> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            pubsub::PubSub::Items(Items {
                max_items: None,
                node: NodeName(node.as_ref().to_string()),
                subid: None,
                items: item_ids
                    .into_iter()
                    .map(|id| {
                        Item(PubSubItem {
                            id: Some(ItemId(id.as_ref().to_string())),
                            publisher: None,
                            payload: None,
                        })
                    })
                    .collect(),
            }),
        );

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        let pubsub::PubSub::Items(items) = pubsub::PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(items.items.into_iter().map(|item| item.0).collect())
    }

    pub async fn load_all_items(
        &self,
        node: impl AsRef<str>,
    ) -> Result<Vec<PubSubItem>, RequestError> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            pubsub::PubSub::Items(Items::new(node.as_ref())),
        );

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        let pubsub::PubSub::Items(items) = pubsub::PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(items.items.into_iter().map(|item| item.0).collect())
    }

    pub async fn delete_items_with_ids<ID: AsRef<str>>(
        &self,
        node: impl AsRef<str>,
        item_ids: impl IntoIterator<Item = ID>,
    ) -> Result<(), RequestError> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Retract(Retract {
                node: NodeName(node.as_ref().to_string()),
                notify: Default::default(),
                items: item_ids
                    .into_iter()
                    .map(|id| {
                        Item(PubSubItem {
                            id: Some(ItemId(id.as_ref().to_string())),
                            publisher: None,
                            payload: None,
                        })
                    })
                    .collect(),
            }),
        );

        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn delete_all_items(&self, node: impl AsRef<str>) -> Result<(), RequestError> {
        let iq = Iq {
            from: None,
            to: None,
            id: self.ctx.generate_id(),
            payload: IqType::Set(
                Element::builder("pubsub", ns::PUBSUB_OWNER)
                    .append(Element::builder("purge", ns::PUBSUB_OWNER).attr("node", node.as_ref()))
                    .build(),
            ),
        };

        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn delete_node(&self, node: impl AsRef<str>) -> Result<(), RequestError> {
        let iq = Iq {
            from: None,
            to: None,
            id: self.ctx.generate_id(),
            payload: IqType::Set(
                Element::builder("pubsub", ns::PUBSUB_OWNER)
                    .append(
                        Element::builder("delete", ns::PUBSUB_OWNER).attr("node", node.as_ref()),
                    )
                    .build(),
            ),
        };
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn request_node_configuration_form(
        &self,
        node: impl AsRef<str>,
    ) -> Result<Option<DataForm>, RequestError> {
        let iq = Iq {
            from: None,
            to: None,
            id: self.ctx.generate_id(),
            payload: IqType::Get(
                Element::builder("pubsub", ns::PUBSUB_OWNER)
                    .append(
                        Element::builder("configure", ns::PUBSUB_OWNER).attr("node", node.as_ref()),
                    )
                    .build(),
            ),
        };

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        response.expect_is("pubsub", ns::PUBSUB_OWNER)?;

        let configure = response
            .get_child("configure", ns::PUBSUB_OWNER)
            .cloned()
            .map(Configure::try_from)
            .transpose()?
            .ok_or(RequestError::UnexpectedResponse)?;

        Ok(Some(
            configure.form.ok_or(RequestError::UnexpectedResponse)?,
        ))
    }
}
