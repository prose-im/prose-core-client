use crate::error::{Error, Result};
use crate::extensions::xmpp_connection_context::IQKind::{Get, Set};
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types::namespace::Namespace;
use crate::types::profile::avatar_data::AvatarData;
use crate::types::profile::avatar_metadata::AvatarMetadata;
use crate::types::profile::image_file::Image;
use crate::types::pubsub::{Item, Items};
use jid::BareJid;
use libstrophe::Stanza;
use std::ops::Deref;
use std::sync::Arc;

pub struct Profile {
    ctx: Arc<XMPPExtensionContext>,
}

impl Profile {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        Profile { ctx }
    }
}

impl XMPPExtension for Profile {
    fn handle_connect(&self) -> Result<()> {
        // Subscribe to avatar changes
        let mut subscribe = Stanza::new_with_name("subscribe", None)?;
        subscribe.set_node(Namespace::AvatarMetadata)?;
        subscribe.set_attribute("jid", self.ctx.jid.to_string())?;

        let mut pubsub = Stanza::new_pubsub()?;
        pubsub.add_child(subscribe)?;

        let mut iq = Stanza::new_iq(Some("set"), None);
        iq.add_child(pubsub)?;

        self.ctx.send_stanza(iq)
    }

    fn handle_pubsub_event(&self, from: &BareJid, node: &str, items: &Stanza) -> Result<()> {
        if node != Namespace::AvatarMetadata {
            return Ok(());
        }

        let items = Items::<AvatarMetadata>::try_from(items)?;
        let first_item = items
            .into_iter()
            .next()
            .unwrap_or(Item::<AvatarMetadata>::new(
                None,
                AvatarMetadata::new(vec![]),
            ));

        self.ctx
            .observer
            .did_receive_updated_avatar_metadata(from.clone(), first_item.value.into_inner());

        Ok(())
    }
}

impl Profile {
    pub fn set_avatar_image(&self, request_id: impl AsRef<str>, image: Image) -> Result<()> {
        let mut data = Stanza::new_with_name("data", Some(Namespace::AvatarData))?;
        data.add_child(Stanza::new_text_node(image.base64_string())?)?;

        let sha1 = image.sha1_checksum();

        let mut item = Stanza::new_with_name("item", None)?;
        item.set_id(&sha1)?;
        item.add_child(data)?;

        let mut publish = Stanza::new_with_name("publish", None)?;
        publish.set_node(Namespace::AvatarData)?;
        publish.add_child(item)?;

        let mut pubsub = Stanza::new_pubsub()?;
        pubsub.add_child(publish)?;

        let ctx = self.ctx.clone();
        let request_id = request_id.as_ref().to_string();

        self.ctx.send_iq(
            Set,
            None,
            pubsub,
            Box::new(move |_| {
                let inner_ctx = ctx.clone();
                ctx.send_iq(
                    Set,
                    None,
                    image.node_for_publishing_metadata(&sha1)?,
                    Box::new(move |_| {
                        inner_ctx.observer.did_set_avatar_image(request_id, sha1);
                        Ok(())
                    }),
                )
            }),
        )
    }

    pub fn load_avatar_image(
        &self,
        request_id: impl AsRef<str>,
        from: &BareJid,
        image_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut item = Stanza::new_with_name("item", None)?;
        item.set_id(image_id)?;

        let mut items = Stanza::new_with_name("items", None)?;
        items.set_node(Namespace::AvatarData)?;
        items.add_child(item)?;

        let mut pubsub = Stanza::new_pubsub()?;
        pubsub.add_child(items)?;

        let ctx = self.ctx.clone();
        let request_id = request_id.as_ref().to_string();
        let from = from.clone();

        self.ctx.send_iq(
            Get,
            Some(&from.to_string()),
            pubsub,
            Box::new(move |result| {
                let payload = match result {
                    Ok(payload) => payload,
                    Err(_) => {
                        ctx.observer.did_load_avatar_image(request_id, from, None);
                        return Ok(());
                    }
                };

                let items_node = match payload.get_child_by_name("items") {
                    Some(node) => node,
                    None => return Ok(()),
                };

                if items_node.get_attribute("node") != Some(Namespace::AvatarData) {
                    return Ok(());
                }

                let mut items = Items::<AvatarData>::try_from(items_node.deref())?;
                ctx.observer
                    .did_load_avatar_image(request_id, from, items.pop().map(|i| i.value));
                Ok(())
            }),
        )
    }

    pub fn load_latest_avatar_metadata(
        &self,
        request_id: impl AsRef<str>,
        from: &BareJid,
    ) -> Result<()> {
        let mut items = Stanza::new_with_name("items", None)?;
        items.set_node(Namespace::AvatarMetadata)?;
        items.set_attribute("max_items", "1")?;

        let mut pubsub = Stanza::new_pubsub()?;
        pubsub.add_child(items)?;

        let ctx = self.ctx.clone();
        let request_id = request_id.as_ref().to_string();
        let from = from.clone();

        self.ctx.send_iq(
            Get,
            Some(&from.to_string()),
            pubsub,
            Box::new(move |result| {
                let payload = match result {
                    Ok(payload) => payload,
                    Err(_) => {
                        ctx.observer
                            .did_load_avatar_metadata(request_id, from, vec![]);
                        return Ok(());
                    }
                };

                let items_node = match payload.get_child_by_name("items") {
                    Some(node) => node,
                    None => return Ok(()),
                };

                if items_node.get_attribute("node") != Some(Namespace::AvatarMetadata) {
                    return Ok(());
                }

                let mut items = Items::<AvatarMetadata>::try_from(items_node.deref())?;
                ctx.observer.did_load_avatar_metadata(
                    request_id,
                    from,
                    items.pop().map(|i| i.value.into_inner()).unwrap_or(vec![]),
                );
                Ok(())
            }),
        )
    }
}

impl Image {
    fn node_for_publishing_metadata(&self, id: impl AsRef<str>) -> Result<Stanza, Error> {
        let mut info = Stanza::new_with_name("info", None)?;
        info.set_attribute("bytes", &self.data.len().to_string())?;
        info.set_attribute("width", &self.width.to_string())?;
        info.set_attribute("height", &self.height.to_string())?;
        info.set_attribute("type", &self.mime_type)?;
        info.set_id(&id)?;

        let mut metadata = Stanza::new_with_name("metadata", Some(Namespace::AvatarMetadata))?;
        metadata.add_child(info)?;

        let mut item = Stanza::new_with_name("item", None)?;
        item.set_id(&id)?;
        item.add_child(metadata)?;

        let mut publish = Stanza::new_with_name("publish", None)?;
        publish.set_node(Namespace::AvatarMetadata)?;
        publish.add_child(item)?;

        let mut pubsub = Stanza::new_pubsub()?;
        pubsub.add_child(publish)?;

        Ok(pubsub)
    }
}
