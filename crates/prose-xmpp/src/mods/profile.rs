use anyhow::Result;
use jid::{BareJid, Jid};
use minidom::Element;
use xmpp_parsers::hashes::Sha1HexAttribute;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::presence::Presence;
use xmpp_parsers::pubsub;
use xmpp_parsers::pubsub::pubsub::Items;
use xmpp_parsers::pubsub::{NodeName, PubSub, PubSubEvent};

use crate::client::ModuleContext;
use crate::event::Event;
use crate::mods::Module;
use crate::ns;
use crate::stanza::VCard4;
use crate::stanza::{avatar, PubSubMessage};
use crate::util::RequestError;

#[derive(Default, Clone)]
pub struct Profile {
    ctx: ModuleContext,
}

impl Module for Profile {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context;
    }

    fn handle_presence_stanza(&self, stanza: &Presence) -> Result<()> {
        self.ctx.schedule_event(Event::Presence(stanza.clone()));
        Ok(())
    }

    fn handle_pubsub_message(&self, pubsub: &PubSubMessage) -> Result<()> {
        for event in pubsub.events.iter() {
            self.handle_pubsub_event(&pubsub.from, event)?
        }
        Ok(())
    }
}

impl Profile {
    fn handle_pubsub_event(&self, from: &Jid, event: &PubSubEvent) -> Result<()> {
        let PubSubEvent::PublishedItems { node, items } = event else {
            return Ok(());
        };

        match node.0.as_ref() {
            ns::VCARD4 => {
                let Some(item) = items.first() else {
                    return Ok(());
                };
                let Some(payload) = &item.payload else {
                    return Ok(());
                };
                let vcard = VCard4::try_from(payload.clone())?;
                self.ctx.schedule_event(Event::Vcard {
                    from: from.clone(),
                    vcard,
                });
            }
            ns::AVATAR_METADATA => {
                let Some(item) = items.first() else {
                    return Ok(());
                };
                let Some(payload) = &item.payload else {
                    return Ok(());
                };
                let metadata = avatar::Metadata::try_from(payload.clone())?;
                self.ctx.schedule_event(Event::AvatarMetadata {
                    from: from.clone(),
                    metadata,
                });
            }
            _ => (),
        }
        Ok(())
    }
}

impl Profile {
    pub async fn load_vcard(&self, from: impl Into<BareJid>) -> Result<Option<VCard4>> {
        let iq = Iq {
            from: None,
            to: None,
            id: self.ctx.generate_id(),
            payload: IqType::Get(
                Element::builder("vcard", ns::VCARD4)
                    .attr("from", from.into().to_string())
                    .build(),
            ),
        };

        let vcard = match self.ctx.send_iq(iq).await {
            Ok(Some(payload)) => VCard4::try_from(payload)?,
            Ok(None) => return Err(RequestError::UnexpectedResponse.into()),
            Err(e) if e.is_item_not_found_err() => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        Ok(Some(vcard))
    }

    pub async fn set_vcard(&self, vcard: VCard4) -> Result<()> {
        let mut iq = Iq::from_set(self.ctx.generate_id(), vcard);
        iq.to = Some(self.ctx.bare_jid().into());
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn delete_vcard(&self) -> Result<()> {
        let mut iq = Iq::from_set(self.ctx.generate_id(), VCard4::new());
        iq.to = Some(self.ctx.bare_jid().into());
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn publish_vcard(&self, vcard: VCard4) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::VCARD4.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: Some(pubsub::ItemId(self.ctx.bare_jid().to_string())),
                        publisher: None,
                        payload: Some(vcard.into()),
                    })],
                },
                publish_options: None,
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn unpublish_vcard(&self) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Retract(pubsub::pubsub::Retract {
                node: NodeName(ns::VCARD4.to_string()),
                notify: Default::default(),
                items: vec![pubsub::pubsub::Item(pubsub::Item {
                    id: Some(pubsub::ItemId(self.ctx.bare_jid().to_string())),
                    publisher: None,
                    payload: None,
                })],
            }),
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn load_latest_avatar_metadata(
        &self,
        from: impl Into<Jid>,
    ) -> Result<Option<avatar::Info>> {
        let iq = Iq {
            from: None,
            to: Some(from.into()),
            id: self.ctx.generate_id(),
            payload: IqType::Get(
                PubSub::Items(Items {
                    max_items: Some(1),
                    node: NodeName(ns::AVATAR_METADATA.to_string()),
                    subid: None,
                    items: vec![],
                })
                .into(),
            ),
        };

        let response = match self.ctx.send_iq(iq).await {
            Ok(iq) => iq,
            Err(e) if e.is_item_not_found_err() => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        .ok_or(RequestError::UnexpectedResponse)?;

        let PubSub::Items(mut items) = PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        if items.items.is_empty() {
            return Ok(None);
        }

        let Some(payload) = items.items.swap_remove(0).payload.take() else {
            return Ok(None);
        };

        let mut metadata = avatar::Metadata::try_from(payload)?;

        if metadata.infos.is_empty() {
            return Ok(None);
        }

        Ok(Some(metadata.infos.swap_remove(0)))
    }

    pub async fn set_avatar_metadata(
        &self,
        bytes_len: usize,
        checksum: &avatar::ImageId,
        mime_type: impl Into<String>,
        width: impl Into<Option<u32>>,
        height: impl Into<Option<u32>>,
    ) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::AVATAR_METADATA.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: Some(pubsub::ItemId(checksum.to_string())),
                        publisher: None,
                        payload: Some(
                            avatar::Metadata {
                                infos: vec![avatar::Info {
                                    bytes: bytes_len as u32,
                                    width: width.into(),
                                    height: height.into(),
                                    id: checksum.clone(),
                                    r#type: mime_type.into(),
                                    url: None,
                                }],
                            }
                            .into(),
                        ),
                    })],
                },
                publish_options: None,
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn load_avatar_image(
        &self,
        from: impl Into<Jid>,
        image_id: &Sha1HexAttribute,
    ) -> Result<Option<Vec<u8>>> {
        let iq = Iq {
            from: None,
            to: Some(from.into()),
            id: self.ctx.generate_id(),
            payload: IqType::Get(
                PubSub::Items(Items {
                    max_items: Some(1),
                    node: NodeName(ns::AVATAR_DATA.to_string()),
                    subid: None,
                    items: vec![pubsub::pubsub::Item(xmpp_parsers::pubsub::Item {
                        id: Some(pubsub::ItemId(image_id.to_hex())),
                        publisher: None,
                        payload: None,
                    })],
                })
                .into(),
            ),
        };

        let response = self
            .ctx
            .send_iq(iq)
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        let PubSub::Items(mut items) = PubSub::try_from(response)? else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        if items.items.is_empty() {
            return Ok(None);
        }

        let Some(payload) = items.items.swap_remove(0).payload.take() else {
            return Ok(None);
        };

        Ok(Some(avatar::Data::try_from(payload)?.data))
    }

    pub async fn set_avatar_image(
        &self,
        checksum: &avatar::ImageId,
        base64_image_data: impl Into<String>,
    ) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            pubsub::PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::AVATAR_DATA.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: Some(pubsub::ItemId(checksum.to_string())),
                        publisher: None,
                        payload: Some(
                            Element::builder("data", ns::AVATAR_DATA)
                                .append(base64_image_data.into())
                                .build(),
                        ),
                    })],
                },
                publish_options: None,
            },
        );

        self.ctx.send_iq(iq).await?;
        Ok(())
    }
}
