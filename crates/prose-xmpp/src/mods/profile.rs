// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;

use anyhow::{bail, Result};
use base64::{engine::general_purpose, DecodeError, Engine as _};
use chrono::{DateTime, FixedOffset};
use jid::{BareJid, Jid};
use minidom::Element;
use sha1::{Digest, Sha1};
use xmpp_parsers::hashes::Sha1HexAttribute;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::pubsub;
use xmpp_parsers::pubsub::pubsub::{Items, PublishOptions};
use xmpp_parsers::pubsub::{NodeName, PubSub, PubSubEvent};
use xmpp_parsers::time::{TimeQuery, TimeResult};
use xmpp_parsers::version::{VersionQuery, VersionResult};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::stanza::avatar::ImageId;
use crate::stanza::last_activity::LastActivityResponse;
use crate::stanza::{avatar, VCard};
use crate::stanza::{LastActivityRequest, VCard4};
use crate::util::{PubSubItemsExt, PubSubQuery, RequestError};
use crate::{ns, ParseError};

#[derive(Default, Clone)]
pub struct Profile {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Vcard {
        from: Jid,
        vcard: VCard4,
    },
    AvatarMetadata {
        from: Jid,
        metadata: avatar::Metadata,
    },
    /// XEP-0202: Entity Time
    EntityTimeQuery {
        from: Jid,
        id: String,
    },
    /// XEP-0092: Software Version
    SoftwareVersionQuery {
        from: Jid,
        id: String,
    },
    /// XEP-0012: Last Activity
    LastActivityQuery {
        from: Jid,
        id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AvatarData {
    Base64(String),
    Data(Box<[u8]>),
}

impl AvatarData {
    pub fn data(&self) -> std::result::Result<Cow<Box<[u8]>>, DecodeError> {
        match self {
            AvatarData::Base64(base64) => Ok(Cow::Owned(
                general_purpose::STANDARD.decode(base64)?.into_boxed_slice(),
            )),
            AvatarData::Data(data) => Ok(Cow::Borrowed(data)),
        }
    }

    pub fn base64(&self) -> Cow<str> {
        match self {
            AvatarData::Base64(base64) => Cow::Borrowed(base64),
            AvatarData::Data(data) => Cow::Owned(general_purpose::STANDARD.encode(data)),
        }
    }

    pub fn generate_sha1_checksum(&self) -> std::result::Result<ImageId, DecodeError> {
        let mut hasher = Sha1::new();
        hasher.update(self.data()?.as_ref());
        Ok(format!("{:x}", hasher.finalize()).into())
    }
}

impl Module for Profile {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context;
    }

    fn handle_iq_stanza(&self, stanza: &Iq) -> Result<()> {
        let IqType::Get(payload) = &stanza.payload else {
            return Ok(());
        };

        // Respond to XEP-0202: Entity Time request
        if payload.is("time", ns::TIME) {
            let Some(from) = &stanza.from else {
                bail!("Missing 'from' in entity time request.")
            };
            self.ctx
                .schedule_event(ClientEvent::Profile(Event::EntityTimeQuery {
                    from: from.clone(),
                    id: stanza.id.clone(),
                }))
        } else if payload.is("query", ns::VERSION) {
            let Some(from) = &stanza.from else {
                bail!("Missing 'from' in software version request.")
            };
            self.ctx
                .schedule_event(ClientEvent::Profile(Event::SoftwareVersionQuery {
                    from: from.clone(),
                    id: stanza.id.clone(),
                }))
        } else if payload.is("query", ns::LAST_ACTIVITY) {
            let Some(from) = &stanza.from else {
                bail!("Missing 'from' in last activity request.")
            };
            self.ctx
                .schedule_event(ClientEvent::Profile(Event::LastActivityQuery {
                    from: from.clone(),
                    id: stanza.id.clone(),
                }))
        }

        Ok(())
    }

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
                self.ctx.schedule_event(ClientEvent::Profile(Event::Vcard {
                    from: from.clone(),
                    vcard,
                }));
            }
            ns::AVATAR_METADATA => {
                let Some(item) = items.first() else {
                    return Ok(());
                };
                let Some(payload) = &item.payload else {
                    return Ok(());
                };
                let metadata = avatar::Metadata::try_from(payload.clone())?;
                self.ctx
                    .schedule_event(ClientEvent::Profile(Event::AvatarMetadata {
                        from: from.clone(),
                        metadata,
                    }));
            }
            _ => (),
        }
        Ok(())
    }
}

impl Profile {
    pub async fn load_vcard4(
        &self,
        from: impl Into<BareJid>,
    ) -> Result<Option<VCard4>, RequestError> {
        let vcard = self
            .ctx
            .query_pubsub_node(
                PubSubQuery::new(self.ctx.generate_id(), ns::VCARD4).set_to(from.into()),
            )
            .await?
            .and_then(|mut items| items.pop())
            .and_then(|item| item.payload)
            .map(VCard4::try_from)
            .transpose()?;
        Ok(vcard)
    }

    pub async fn load_vcard_temp(
        &self,
        from: impl Into<Jid>,
    ) -> Result<Option<VCard>, RequestError> {
        let iq = Iq {
            from: None,
            to: Some(from.into()),
            id: self.ctx.generate_id(),
            payload: IqType::Get(Element::builder("vCard", ns::VCARD).build()),
        };

        let vcard = match self.ctx.send_iq(iq).await {
            Ok(Some(payload)) => {
                VCard::try_from(payload).map_err(|e| ParseError::Generic { msg: e.to_string() })?
            }
            Ok(None) => return Err(RequestError::UnexpectedResponse.into()),
            Err(e) if e.is_item_not_found_err() => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        Ok(Some(vcard))
    }

    pub async fn publish_vcard_temp(&self, vcard: VCard) -> Result<()> {
        let mut iq = Iq::from_set(self.ctx.generate_id(), vcard);
        iq.to = Some(self.ctx.bare_jid().into());
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn publish_vcard4(
        &self,
        vcard: VCard4,
        publish_options: Option<PublishOptions>,
    ) -> Result<()> {
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
                publish_options,
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn publish_nickname(&self, nickname: Option<String>) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            PubSub::Publish {
                publish: pubsub::pubsub::Publish {
                    node: NodeName(ns::NICK.to_string()),
                    items: vec![pubsub::pubsub::Item(pubsub::Item {
                        id: None,
                        publisher: None,
                        payload: Some(
                            Element::builder("nick", ns::NICK)
                                .append_all(nickname)
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

    pub async fn delete_vcard(&self) -> Result<()> {
        let mut iq = Iq::from_set(self.ctx.generate_id(), VCard4::new());
        iq.to = Some(self.ctx.bare_jid().into());
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
        from: &BareJid,
    ) -> Result<Option<avatar::Info>, RequestError> {
        let metadata = self
            .ctx
            .query_pubsub_node(
                PubSubQuery::new(self.ctx.generate_id(), ns::AVATAR_METADATA)
                    .set_to(from.clone())
                    .set_max_items(1),
            )
            .await?
            .unwrap_or_default()
            .find_first_payload::<avatar::Metadata>("metadata", ns::AVATAR_METADATA)?;

        let Some(mut metadata) = metadata else {
            return Ok(None);
        };

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
    ) -> Result<Option<AvatarData>, RequestError> {
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

        Ok(Some(AvatarData::Base64(payload.text())))
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

    /// XEP-0202: Entity Time
    /// https://xmpp.org/extensions/xep-0202.html
    pub async fn load_entity_time(
        &self,
        from: impl Into<Jid>,
    ) -> Result<DateTime<FixedOffset>, RequestError> {
        let response = self
            .ctx
            .send_iq(Iq::from_get(self.ctx.generate_id(), TimeQuery).with_to(from.into()))
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(TimeResult::try_from(response)?.0 .0)
    }

    /// XEP-0202: Entity Time
    /// https://xmpp.org/extensions/xep-0202.html
    pub async fn load_server_time(&self) -> Result<DateTime<FixedOffset>> {
        let response = self
            .ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), TimeQuery)
                    .with_to(self.ctx.server_jid().into()),
            )
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(TimeResult::try_from(response)?.0 .0)
    }

    /// XEP-0012: Last Activity
    /// https://xmpp.org/extensions/xep-0012.html
    pub async fn load_last_activity(
        &self,
        from: impl Into<Jid>,
    ) -> Result<LastActivityResponse, RequestError> {
        let response = self
            .ctx
            .send_iq(Iq::from_get(self.ctx.generate_id(), LastActivityRequest).with_to(from.into()))
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(LastActivityResponse::try_from(response)?)
    }

    /// XEP-0202: Entity Time
    /// https://xmpp.org/extensions/xep-0202.html
    pub async fn send_entity_time_response(
        &self,
        time: DateTime<FixedOffset>,
        to: Jid,
        id: impl AsRef<str>,
    ) -> Result<()> {
        let response = Iq::from_result(
            id.as_ref(),
            Some(TimeResult(xmpp_parsers::date::DateTime(time))),
        )
        .with_to(to);
        self.ctx.send_stanza(response)?;
        Ok(())
    }

    /// XEP-0092: Software Version
    /// https://xmpp.org/extensions/xep-0092.html
    pub async fn load_software_version(&self, from: impl Into<Jid>) -> Result<VersionResult> {
        let response = self
            .ctx
            .send_iq(Iq::from_get(self.ctx.generate_id(), VersionQuery).with_to(from.into()))
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(VersionResult::try_from(response)?)
    }

    /// XEP-0092: Software Version
    /// https://xmpp.org/extensions/xep-0092.html
    pub async fn send_software_version_response(
        &self,
        software_version: VersionResult,
        to: Jid,
        id: impl AsRef<str>,
    ) -> Result<()> {
        self.ctx
            .send_stanza(Iq::from_result(id.as_ref(), Some(software_version)).with_to(to))?;
        Ok(())
    }

    /// XEP-0012: Last Activity
    /// https://xmpp.org/extensions/xep-0012.html
    pub async fn send_last_activity_response(
        &self,
        seconds: u64,
        status: Option<String>,
        to: Jid,
        id: impl AsRef<str>,
    ) -> Result<()> {
        self.ctx.send_stanza(
            Iq::from_result(id.as_ref(), Some(LastActivityResponse { seconds, status }))
                .with_to(to),
        )?;
        Ok(())
    }
}
