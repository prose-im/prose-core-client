// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use tracing::debug;
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::{profile, status};
use prose_xmpp::stanza::{avatar, UserActivity, VCard4};
use prose_xmpp::Event;

use crate::app::deps::{
    DynAppServiceDependencies, DynAvatarRepository, DynUserInfoRepository, DynUserProfileRepository,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::user_info::models::{
    AvatarMetadata, Presence as DomainPresence, UserActivity as DomainUserActivity,
};
use crate::ClientEvent;

pub struct UserStateEventHandler {
    deps: DynAppServiceDependencies,
    avatar_repository: DynAvatarRepository,
    user_info_repository: DynUserInfoRepository,
    user_profile_repository: DynUserProfileRepository,
}

#[async_trait]
impl XMPPEventHandler for UserStateEventHandler {
    fn name(&self) -> &'static str {
        "user_state"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Status(event) => match event {
                status::Event::Presence(presence) => {
                    self.presence_did_change(&presence).await?;
                    // Since presence can contain more information than we handle, give others
                    // a chance to handle this event has well…
                    Ok(Some(Event::Status(status::Event::Presence(presence))))
                }
                status::Event::UserActivity {
                    from,
                    user_activity,
                } => {
                    self.user_activity_did_change(from, user_activity).await?;
                    Ok(None)
                }
            },
            Event::Profile(event) => match event {
                profile::Event::Vcard { from, vcard } => {
                    self.vcard_did_change(from, vcard).await?;
                    Ok(None)
                }
                profile::Event::AvatarMetadata { from, metadata } => {
                    self.avatar_metadata_did_change(from, metadata).await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Profile(event))),
            },
            _ => Ok(Some(event)),
        }
    }
}

impl UserStateEventHandler {
    async fn presence_did_change(&self, presence: &Presence) -> Result<()> {
        let Some(from) = &presence.from else {
            return Ok(());
        };

        self.user_info_repository
            .set_user_presence(from, &DomainPresence::from(presence.clone()))
            .await?;

        self.deps
            .event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged {
                jid: from.to_bare(),
            });

        Ok(())
    }

    async fn vcard_did_change(&self, from: Jid, vcard: VCard4) -> Result<()> {
        debug!("New vcard for {} {:?}", from, vcard);

        let from = from.into_bare();
        self.user_profile_repository
            .set(&from, &vcard.try_into()?)
            .await?;
        self.deps
            .event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged { jid: from });

        Ok(())
    }

    async fn avatar_metadata_did_change(
        &self,
        from: Jid,
        metadata: avatar::Metadata,
    ) -> Result<()> {
        debug!("New metadata for {} {:?}", from, metadata);

        let Some(metadata) = metadata
            .infos
            .first()
            .map(|i| AvatarMetadata::from(i.clone()))
        else {
            return Ok(());
        };

        let from = from.into_bare();

        self.user_info_repository
            .set_avatar_metadata(&from, &metadata)
            .await?;
        self.avatar_repository
            .precache_avatar_image(&from, &metadata.to_info())
            .await?;

        self.deps
            .event_dispatcher
            .dispatch_event(ClientEvent::AvatarChanged { jid: from });

        Ok(())
    }

    async fn user_activity_did_change(&self, from: Jid, user_activity: UserActivity) -> Result<()> {
        let jid = from.into_bare();
        let user_activity = DomainUserActivity::try_from(user_activity)?;
        self.user_info_repository
            .set_user_activity(&jid, Some(&user_activity))
            .await?;
        self.deps
            .event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged { jid });
        Ok(())
    }
}
