use std::sync::Arc;

use jid::{BareJid, Jid};

use crate::modules::profile::avatar::ImageId;
use crate::modules::profile::types::avatar::{Info, Metadata};
use crate::modules::profile::VCard;
use crate::modules::Module;
use crate::stanza::iq::Kind::{Get, Set};
use crate::stanza::pubsub::{Event, Item, Items, Publish, Retract};
use crate::stanza::{presence, Namespace, Presence, PubSub, Stanza, StanzaBase, IQ};

use super::super::Context;

pub trait ProfileDelegate: Send + Sync {
    fn vcard_did_change(&self, from: &Jid, vcard: &VCard);
    fn avatar_metadata_did_change(&self, from: &Jid, metadata: &Metadata);
    fn presence_did_change(&self, from: &Jid, presence: &Presence);
}

pub struct Profile {
    delegate: Option<Arc<dyn ProfileDelegate>>,
}

impl Profile {
    pub fn new(delegate: Option<Arc<dyn ProfileDelegate + 'static>>) -> Self {
        Profile { delegate }
    }
}

impl Module for Profile {
    fn handle_presence_stanza(&self, _ctx: &Context, stanza: &Presence) -> anyhow::Result<()> {
        let Some(handler) = &self.delegate else {
            return Ok(())
        };

        let Some(from) = stanza.from() else {
            return Ok(())
        };

        handler.presence_did_change(&from, stanza);
        Ok(())
    }

    fn handle_pubsub_event(
        &self,
        _ctx: &Context,
        from: &Jid,
        node: &Namespace,
        event: &Event,
    ) -> anyhow::Result<()> {
        let Some(handler) = &self.delegate else {
            return Ok(())
        };

        match node {
            &Namespace::VCard => {
                let Some(items) = event.items() else {
                    return Ok(())
                };
                let Some(item) = items.first_child() else {
                    return Ok(())
                };
                let Some(vcard) = item.first_child().map(Into::<VCard>::into) else {
                    return Ok(())
                };
                handler.vcard_did_change(from, &vcard);
            }
            &Namespace::AvatarMetadata => {
                let Some(items) = event.items() else {
                    return Ok(())
                };
                let Some(item) = items.first_child() else {
                    return Ok(())
                };
                let Some(metadata) = item.first_child().map(Into::<Metadata>::into) else {
                    return Ok(())
                };
                handler.avatar_metadata_did_change(from, &metadata);
            }
            _ => (),
        }
        Ok(())
    }
}

impl Profile {
    pub async fn load_vcard(
        &self,
        ctx: &Context<'_>,
        from: BareJid,
    ) -> anyhow::Result<Option<VCard>> {
        let iq = IQ::new(Get, ctx.generate_id())
            .add_child(Stanza::new("vcard").set_namespace(Namespace::VCard))
            .set_to(from);

        let vcard: Option<VCard> = ctx
            .send_iq(iq)
            .await?
            .child_by_name_and_namespace("vcard", Namespace::VCard)
            .map(|s| s.clone().into());

        Ok(vcard)
    }

    pub async fn set_vcard(&self, ctx: &Context<'_>, vcard: VCard<'_>) -> anyhow::Result<()> {
        ctx.send_iq(
            IQ::new(Set, ctx.generate_id())
                .add_child(vcard)
                .set_to(BareJid::from(ctx.jid.clone())),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_vcard(&self, ctx: &Context<'_>) -> anyhow::Result<()> {
        ctx.send_iq(
            IQ::new(Set, ctx.generate_id())
                .add_child(VCard::new())
                .set_to(BareJid::from(ctx.jid.clone())),
        )
        .await?;
        Ok(())
    }

    pub async fn publish_vcard(&self, ctx: &Context<'_>, vcard: VCard<'_>) -> anyhow::Result<()> {
        let iq = IQ::new(Set, ctx.generate_id()).add_child(
            PubSub::new().set_publish(
                Publish::new()
                    .set_node(Namespace::VCard.to_string())
                    .set_item(Item::new(BareJid::from(ctx.jid.clone())).add_child(vcard)),
            ),
        );
        ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn unpublish_vcard(&self, ctx: &Context<'_>) -> anyhow::Result<()> {
        let iq = IQ::new(Set, ctx.generate_id()).add_child(
            PubSub::new().set_retract(
                Retract::new()
                    .set_node(Namespace::VCard.to_string())
                    .set_item(Item::new(BareJid::from(ctx.jid.clone()))),
            ),
        );
        ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn load_latest_avatar_metadata(
        &self,
        ctx: &Context<'_>,
        from: impl Into<Jid>,
    ) -> anyhow::Result<Option<Info>> {
        let iq = IQ::new(Get, ctx.generate_id()).set_to(from).add_child(
            PubSub::new().set_items(
                Items::new()
                    .set_node(Namespace::AvatarMetadata.to_string())
                    .set_max_items(1),
            ),
        );

        let response = ctx.send_iq(iq).await?;

        let Some(pubsub) = response.pubsub() else {
            return Ok(None)
        };

        Ok(pubsub.avatar_metadata_info())
    }

    pub async fn set_avatar_metadata(
        &self,
        ctx: &Context<'_>,
        bytes_len: usize,
        checksum: &ImageId,
        mime_type: impl AsRef<str>,
        width: impl Into<Option<u32>>,
        height: impl Into<Option<u32>>,
    ) -> anyhow::Result<()> {
        let iq = IQ::new(Set, ctx.generate_id()).add_child(
            PubSub::new().set_publish(
                Publish::new()
                    .set_node(Namespace::AvatarMetadata.to_string())
                    .set_item(
                        Item::new(checksum.as_ref()).add_child(Metadata::new().set_info(
                            Info::new(bytes_len as i64, checksum, mime_type, width, height),
                        )),
                    ),
            ),
        );
        ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn load_avatar_image(
        &self,
        ctx: &Context<'_>,
        from: impl Into<Jid>,
        image_id: &ImageId,
    ) -> anyhow::Result<Option<String>> {
        let iq = IQ::new(Get, ctx.generate_id()).set_to(from).add_child(
            PubSub::new().set_items(
                Items::new()
                    .set_node(Namespace::AvatarData.to_string())
                    .set_items(vec![Item::new(image_id.as_ref())]),
            ),
        );

        let response = ctx.send_iq(iq).await?;
        let Some(pubsub) = response.pubsub() else {
            return Ok(None)
        };

        Ok(pubsub.avatar_image_data())
    }

    pub async fn set_avatar_image(
        &self,
        ctx: &Context<'_>,
        checksum: &ImageId,
        base64_image_data: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let iq = IQ::new(Set, ctx.generate_id()).add_child(
            PubSub::new().set_publish(
                Publish::new()
                    .set_node(Namespace::AvatarData.to_string())
                    .set_item(
                        Item::new(checksum.as_ref()).add_child(
                            Stanza::new_text_node("data", base64_image_data)
                                .set_namespace(Namespace::AvatarData),
                        ),
                    ),
            ),
        );
        ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn send_presence(
        &self,
        ctx: &Context<'_>,
        show: Option<presence::Show>,
        status: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut presence = Presence::new();

        if let Some(show) = show {
            presence = presence.set_show(show);
        }
        if let Some(status) = status {
            presence = presence.set_status(status);
        }

        ctx.send_stanza(presence);
        Ok(())
    }
}

impl<'a> PubSub<'a> {
    fn avatar_metadata_info<'b>(&self) -> Option<Info<'b>> {
        let items = self.items()?;
        let mut items_iter = items.items();
        let first_item = items_iter.next()?;

        let metadata: Metadata = first_item
            .child_by_name_and_namespace("metadata", Namespace::AvatarMetadata)?
            .into();

        metadata.info().map(|i| i.clone())
    }

    fn avatar_image_data(&self) -> Option<String> {
        let items = self.items()?;
        let mut items_iter = items.items();
        let first_item = items_iter.next()?;
        let data = first_item.child_by_name_and_namespace("data", Namespace::AvatarData)?;
        data.text()
    }
}
