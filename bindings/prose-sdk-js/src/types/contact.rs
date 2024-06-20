// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::dtos::{
    Availability as CoreAvailability, Contact as CoreContact, Group as CoreGroup,
    PresenceSubscription as CorePresenceSubscription, UserStatus as CoreUserStatus,
};
use wasm_bindgen::prelude::*;

use super::BareJid;

#[wasm_bindgen]
pub struct Contact(CoreContact);

impl From<CoreContact> for Contact {
    fn from(value: CoreContact) -> Self {
        Contact(value)
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum Availability {
    Available = 0,
    Unavailable = 1,
    DoNotDisturb = 2,
    Away = 3,
    Invisible = 4,
}

#[wasm_bindgen]
pub enum PresenceSubscription {
    /// We have requested to subscribe to the contact's presence, but they haven't approved yet.
    Requested = 0,
    /// Both we and the contact are subscribed to each other's presence.
    Mutual = 1,
    /// The contact is subscribed to our presence, so they can see our status.
    TheyFollow = 2,
    /// We are subscribed to the contact's presence, so we can see their status.
    WeFollow = 3,
    /// There is no presence subscription between us and the contact.
    None = 4,
}

#[wasm_bindgen]
pub enum Group {
    Team = 0,
    Other = 1,
}

impl From<Availability> for CoreAvailability {
    fn from(value: Availability) -> Self {
        match value {
            Availability::Available => CoreAvailability::Available,
            Availability::Unavailable => CoreAvailability::Unavailable,
            Availability::DoNotDisturb => CoreAvailability::DoNotDisturb,
            Availability::Away => CoreAvailability::Away,
            Availability::Invisible => CoreAvailability::Invisible,
        }
    }
}

#[wasm_bindgen]
pub struct UserStatus(CoreUserStatus);

impl From<CoreUserStatus> for UserStatus {
    fn from(value: CoreUserStatus) -> Self {
        UserStatus(value)
    }
}

#[wasm_bindgen]
impl Contact {
    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.id.clone().into_inner().into()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn availability(&self) -> Availability {
        self.0.availability.clone().into()
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<UserStatus> {
        self.0
            .status
            .as_ref()
            .map(|activity| UserStatus(activity.clone()))
    }

    #[wasm_bindgen(getter)]
    pub fn group(&self) -> Group {
        self.0.group.clone().into()
    }

    #[wasm_bindgen(getter, js_name = "presenceSubscription")]
    pub fn presence_subscription(&self) -> PresenceSubscription {
        self.0.presence_subscription.into()
    }

    // pub avatar: Option<String>,
}

#[wasm_bindgen]
impl UserStatus {
    #[wasm_bindgen(constructor)]
    pub fn new(icon: &str, status: Option<String>) -> Self {
        UserStatus(CoreUserStatus {
            emoji: icon.to_string(),
            status: status.clone(),
        })
    }

    #[wasm_bindgen(getter, js_name = "icon")]
    pub fn emoji(&self) -> String {
        self.0.emoji.clone()
    }

    #[wasm_bindgen(getter, js_name = "text")]
    pub fn status(&self) -> Option<String> {
        self.0.status.clone()
    }
}

impl From<CoreAvailability> for Availability {
    fn from(value: CoreAvailability) -> Self {
        match value {
            CoreAvailability::Available => Availability::Available,
            CoreAvailability::Unavailable => Availability::Unavailable,
            CoreAvailability::DoNotDisturb => Availability::DoNotDisturb,
            CoreAvailability::Away => Availability::Away,
            CoreAvailability::Invisible => Availability::Invisible,
        }
    }
}

impl From<CoreGroup> for Group {
    fn from(value: CoreGroup) -> Self {
        match value {
            CoreGroup::Team => Group::Team,
            CoreGroup::Other => Group::Other,
        }
    }
}

impl From<CorePresenceSubscription> for PresenceSubscription {
    fn from(value: CorePresenceSubscription) -> Self {
        match value {
            CorePresenceSubscription::Requested => PresenceSubscription::Requested,
            CorePresenceSubscription::Mutual => PresenceSubscription::Mutual,
            CorePresenceSubscription::TheyFollow => PresenceSubscription::TheyFollow,
            CorePresenceSubscription::WeFollow => PresenceSubscription::WeFollow,
            CorePresenceSubscription::None => PresenceSubscription::None,
        }
    }
}
