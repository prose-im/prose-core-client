// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::app::services::{
    AccountService, ConnectionService, ContactsService, RoomsService, UserDataService,
};
use crate::client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedDriver};
use crate::ClientEvent;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

pub trait ClientDelegate: SendUnlessWasm + SyncUnlessWasm {
    fn handle_event(&self, client: Client, event: ClientEvent);
}

impl Client {
    pub fn builder() -> ClientBuilder<UndefinedDriver, UndefinedAvatarCache> {
        ClientBuilder::new()
    }
}

pub struct ClientInner {
    pub(crate) connection: ConnectionService,
    pub account: AccountService,
    pub contacts: ContactsService,
    pub rooms: RoomsService,
    pub user_data: UserDataService,
}

impl From<Arc<ClientInner>> for Client {
    fn from(inner: Arc<ClientInner>) -> Self {
        Client { inner }
    }
}

impl Deref for Client {
    type Target = ClientInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Client {
    pub async fn connect(
        &self,
        jid: &BareJid,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.connection.connect(jid, password).await
    }

    pub async fn disconnect(&self) {
        self.connection.disconnect().await
    }

    pub async fn start_observing_rooms(&self) -> Result<()> {
        // TODO!
        Ok(())
    }
}

// }
//
// impl<D: DataCache, A: AvatarCache> Client<D, A> {
//     pub async fn delete_cached_data(&self) -> Result<()> {
//         self.inner.data_cache.delete_all().await?;
//         self.inner.avatar_cache.delete_all_cached_images().await?;
//         Ok(())
//     }
// }
//
// impl<D: DataCache, A: AvatarCache> Client<D, A> {
//     pub async fn start_observing_rooms(&self) -> Result<()> {
//         // if self.inner.is_observing_rooms.swap(true, Ordering::Acquire) {
//         //     return Ok(());
//         // }
//         //
//         // let user_jid = self.connected_jid()?;
//         //
//         // // Insert contacts as "Direct Message" rooms…
//         // *self.inner.connected_rooms.write() = self
//         //     .load_contacts(CachePolicy::default())
//         //     .await?
//         //     .into_iter()
//         //     .map(|contact| {
//         //         (
//         //             contact.jid.clone(),
//         //             RoomEnvelope::from((contact, user_jid.clone(), self)),
//         //         )
//         //     })
//         //     .collect();
//         //
//         // let bookmarks = match self.load_bookmarks().await {
//         //     Ok(bookmarks) => bookmarks,
//         //     Err(error) => {
//         //         error!("Failed to load bookmarks. Reason: {}", error.to_string());
//         //         Default::default()
//         //     }
//         // };
//         // let mut invalid_bookmarks = vec![];
//         //
//         // for (jid, bookmark) in bookmarks.iter() {
//         //     let result = self
//         //         .enter_room(
//         //             &jid,
//         //             bookmark.conference.nick.as_deref(),
//         //             bookmark.conference.password.as_deref(),
//         //         )
//         //         .await;
//         //
//         //     match result {
//         //         Ok(_) => (),
//         //         Err(error) if error.defined_condition() == Some(DefinedCondition::Gone) => {
//         //             // The room does not exist anymore…
//         //             invalid_bookmarks.push(bookmark.jid.to_bare());
//         //         }
//         //         Err(error) => error!(
//         //             "Failed to enter room {}. Reason: {}",
//         //             bookmark.jid,
//         //             error.to_string()
//         //         ),
//         //     }
//         // }
//         //
//         // *self.inner.bookmarks.write() = bookmarks;
//         //
//         // if !invalid_bookmarks.is_empty() {
//         //     self.inner
//         //         .connected_rooms
//         //         .write()
//         //         .retain(|room_jid, _| !invalid_bookmarks.contains(room_jid));
//         //
//         //     info!("Deleting {} invalid bookmarks…", invalid_bookmarks.len());
//         //     if let Err(error) = self
//         //         .remove_and_publish_bookmarks(invalid_bookmarks.as_slice())
//         //         .await
//         //     {
//         //         error!(
//         //             "Failed to delete invalid bookmarks. Reason {}",
//         //             error.to_string()
//         //         )
//         //     }
//         // }
//
//         //self.send_event(ClientEvent::RoomsChanged);
//
//         Ok(())
//     }
//
//     pub async fn load_account_settings(&self) -> Result<AccountSettings> {
//         Ok(self
//             .inner
//             .data_cache
//             .load_account_settings()
//             .await?
//             .unwrap_or_default())
//     }
//
//     pub async fn save_account_settings(&self, settings: &AccountSettings) -> Result<()> {
//         self.inner
//             .data_cache
//             .save_account_settings(settings)
//             .await?;
//         Ok(())
//     }
// }
//
// impl<D: DataCache, A: AvatarCache> Client<D, A> {
//     async fn gather_server_features(&self) -> Result<()> {
//         let caps = self.client.get_mod::<mods::Caps>();
//         let disco_items = caps.query_server_disco_items(None).await?;
//
//         for item in disco_items.items {
//             let info = caps.query_disco_info(item.jid.clone(), None).await?;
//
//             if info
//                 .identities
//                 .iter()
//                 .find(|ident| ident.category == "conference")
//                 .is_none()
//             {
//                 continue;
//             }
//
//             *self.inner.muc_service.write() = Some(muc::Service {
//                 jid: item.jid.into_bare(),
//             });
//             break;
//         }
//
//         Ok(())
//     }
// }
