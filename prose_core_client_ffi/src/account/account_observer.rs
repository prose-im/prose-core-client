// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::forwarded_message::ForwardedMessage;
use crate::types::mam::MAMPreferences;
use crate::types::message::Message;
use crate::types::presence::Presence;
use crate::types::profile::avatar_data::AvatarData;
use crate::types::profile::avatar_metadata::AvatarMetadataInfo;
use crate::types::roster::Roster;
use jid::BareJid;
#[cfg(feature = "test-helpers")]
use std::sync::{Arc, Mutex};

#[cfg_attr(feature = "test-helpers", mockiato::mockable)]
pub trait AccountObserver: Send + Sync {
    fn did_connect(&self);
    fn did_disconnect(&self);

    fn did_receive_message(&self, message: Message);
    fn did_receive_message_carbon(&self, message: ForwardedMessage);
    fn did_receive_sent_message_carbon(&self, message: ForwardedMessage);
    fn did_receive_roster(&self, roster: Roster);
    fn did_receive_presence(&self, presence: Presence);
    fn did_receive_presence_subscription_request(&self, from: BareJid);
    fn did_receive_archiving_preferences(&self, preferences: MAMPreferences);
    fn did_receive_messages_in_chat(
        &self,
        request_id: String,
        jid: BareJid,
        messages: Vec<ForwardedMessage>,
        is_complete: bool,
    );
    fn did_load_avatar_metadata(
        &self,
        request_id: String,
        jid: BareJid,
        metadata: Vec<AvatarMetadataInfo>,
    );
    fn did_load_avatar_image(&self, request_id: String, jid: BareJid, image: Option<AvatarData>);
    fn did_set_avatar_image(&self, request_id: String, image_id: String);
    fn did_receive_updated_avatar_metadata(&self, jid: BareJid, metadata: Vec<AvatarMetadataInfo>);
}

#[cfg(feature = "test-helpers")]
impl<'mock> AccountObserver for Arc<Mutex<AccountObserverMock<'mock>>> {
    fn did_connect(&self) {
        self.lock().unwrap().did_connect();
    }
    fn did_disconnect(&self) {
        self.lock().unwrap().did_disconnect();
    }

    fn did_receive_message(&self, message: Message) {
        self.lock().unwrap().did_receive_message(message);
    }
    fn did_receive_message_carbon(&self, message: ForwardedMessage) {
        self.lock().unwrap().did_receive_message_carbon(message);
    }
    fn did_receive_sent_message_carbon(&self, message: ForwardedMessage) {
        self.lock()
            .unwrap()
            .did_receive_sent_message_carbon(message);
    }
    fn did_receive_roster(&self, roster: Roster) {
        self.lock().unwrap().did_receive_roster(roster);
    }
    fn did_receive_presence(&self, presence: Presence) {
        self.lock().unwrap().did_receive_presence(presence);
    }
    fn did_receive_presence_subscription_request(&self, from: BareJid) {
        self.lock()
            .unwrap()
            .did_receive_presence_subscription_request(from);
    }
    fn did_receive_archiving_preferences(&self, preferences: MAMPreferences) {
        self.lock()
            .unwrap()
            .did_receive_archiving_preferences(preferences);
    }
    fn did_receive_messages_in_chat(
        &self,
        request_id: String,
        jid: BareJid,
        messages: Vec<ForwardedMessage>,
        is_complete: bool,
    ) {
        self.lock()
            .unwrap()
            .did_receive_messages_in_chat(request_id, jid, messages, is_complete);
    }

    fn did_load_avatar_metadata(
        &self,
        request_id: String,
        jid: BareJid,
        metadata: Vec<AvatarMetadataInfo>,
    ) {
        self.lock()
            .unwrap()
            .did_load_avatar_metadata(request_id, jid, metadata);
    }

    fn did_load_avatar_image(&self, request_id: String, jid: BareJid, image: Option<AvatarData>) {
        self.lock()
            .unwrap()
            .did_load_avatar_image(request_id, jid, image);
    }

    fn did_set_avatar_image(&self, request_id: String, image_id: String) {
        self.lock()
            .unwrap()
            .did_set_avatar_image(request_id, image_id);
    }

    fn did_receive_updated_avatar_metadata(&self, jid: BareJid, metadata: Vec<AvatarMetadataInfo>) {
        self.lock()
            .unwrap()
            .did_receive_updated_avatar_metadata(jid, metadata);
    }
}

#[cfg(feature = "test-helpers")]
unsafe impl<'mock> Send for AccountObserverMock<'mock> {}
#[cfg(feature = "test-helpers")]
unsafe impl<'mock> Sync for AccountObserverMock<'mock> {}
