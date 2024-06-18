// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;
use std::sync::OnceLock;

use anyhow::Result;
use jid::Jid;
use minidom::Element;
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::disco::Item as DiscoItem;
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::pubsub::owner::Configure;
use xmpp_parsers::pubsub::pubsub::{Item, Notify, PublishOptions, Retract};
use xmpp_parsers::pubsub::{pubsub, Item as PubSubItem, ItemId, NodeName, PubSubEvent};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::stanza::PubSubMessage;
use crate::util::{PubSubQuery, RequestError};
use crate::{ns, ElementExt};

#[derive(Default, Clone)]
pub struct PubSub {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PubSubMessage { message: PubSubMessage },
}

impl Module for PubSub {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_pubsub_message(&self, pubsub: &PubSubMessage) -> Result<()> {
        let Some(node) = pubsub.events.first().map(|event| event.node()) else {
            return Ok(());
        };

        // TODO: Remove this once elements are consumed by modules.
        static IGNORED_PUBSUB_NODES: OnceLock<HashSet<&str>> = OnceLock::new();
        let ignored_pubsub_nodes = IGNORED_PUBSUB_NODES.get_or_init(|| {
            let mut m = HashSet::new();
            m.insert(ns::AVATAR_METADATA);
            m.insert(ns::BOOKMARKS);
            m.insert(ns::BOOKMARKS2);
            m.insert(ns::USER_ACTIVITY);
            m.insert(ns::VCARD4);
            m
        });

        // Ignore nodes that are handled in other modules…
        if ignored_pubsub_nodes.contains(node.0.as_str()) {
            return Ok(());
        }

        self.ctx
            .schedule_event(ClientEvent::PubSub(Event::PubSubMessage {
                message: pubsub.clone(),
            }));

        Ok(())
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
                rsm: None,
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

    pub async fn load_items_with_ids<ID: Into<String>>(
        &self,
        node: impl AsRef<str>,
        item_ids: impl IntoIterator<Item = ID>,
    ) -> Result<Vec<PubSubItem>, RequestError> {
        let items = self
            .ctx
            .query_pubsub_node(
                PubSubQuery::new(self.ctx.generate_id(), node.as_ref()).set_item_ids(item_ids),
            )
            .await?
            .unwrap_or_default();

        Ok(items)
    }

    pub async fn load_objects_with_ids<T: TryFrom<Element>, ID: Into<String>>(
        &self,
        node: impl AsRef<str>,
        item_ids: impl IntoIterator<Item = ID>,
    ) -> Result<Vec<T>, RequestError>
    where
        T::Error: Into<RequestError>,
    {
        self.load_items_with_ids(node, item_ids)
            .await?
            .into_iter()
            .map(|item| {
                let Some(payload) = item.payload else {
                    return Err(RequestError::UnexpectedResponse);
                };
                return T::try_from(payload).map_err(Into::into);
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn load_all_items(
        &self,
        node: impl AsRef<str>,
    ) -> Result<Vec<PubSubItem>, RequestError> {
        let items = self
            .ctx
            .query_pubsub_node(PubSubQuery::new(self.ctx.generate_id(), node.as_ref()))
            .await?
            .unwrap_or_default();

        Ok(items)
    }

    pub async fn load_all_objects<T: TryFrom<Element>>(
        &self,
        node: impl AsRef<str>,
    ) -> Result<Vec<T>, RequestError>
    where
        T::Error: Into<RequestError>,
    {
        self.load_all_items(node)
            .await?
            .into_iter()
            .map(|item| {
                let Some(payload) = item.payload else {
                    return Err(RequestError::UnexpectedResponse);
                };
                return T::try_from(payload).map_err(Into::into);
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn delete_items_with_ids<ID: AsRef<str>>(
        &self,
        node: impl AsRef<str>,
        item_ids: impl IntoIterator<Item = ID>,
        notify: bool,
    ) -> Result<(), RequestError> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Retract(Retract {
                node: NodeName(node.as_ref().to_string()),
                notify: if notify { Notify::True } else { Notify::False },
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
        self.ctx.delete_pubsub_node(node).await
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

    pub async fn subscribe_to_node(
        &self,
        jid: &Jid,
        node: Option<&str>,
    ) -> Result<pubsub::SubscriptionElem, RequestError> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Subscribe {
                subscribe: Some(pubsub::Subscribe {
                    jid: jid.clone(),
                    node: node.map(|n| NodeName(n.to_string())),
                }),
                options: None,
            },
        );

        let response = pubsub::PubSub::try_from(
            self.ctx
                .send_iq(iq)
                .await?
                .ok_or(RequestError::UnexpectedResponse)?,
        )?;

        let pubsub::PubSub::Subscription(sub) = response else {
            return Err(RequestError::UnexpectedResponse);
        };

        Ok(sub)
    }
}

trait PubSubEventExt {
    fn node(&self) -> &NodeName;
}

impl PubSubEventExt for PubSubEvent {
    fn node(&self) -> &NodeName {
        match self {
            PubSubEvent::Configuration { node, .. } => node,
            PubSubEvent::Delete { node, .. } => node,
            PubSubEvent::PublishedItems { node, .. } => node,
            PubSubEvent::RetractedItems { node, .. } => node,
            PubSubEvent::Purge { node, .. } => node,
            PubSubEvent::Subscription { node, .. } => node,
        }
    }
}
