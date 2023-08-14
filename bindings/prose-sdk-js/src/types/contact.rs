// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::types::{
    roster::Group as CoreGroup, Availability as CoreAvailability, Contact as CoreContact,
    UserActivity as CoreUserActivity,
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
pub enum Availability {
    Available = 0,
    Unavailable = 1,
    DoNotDisturb = 2,
    Away = 3,
}

#[wasm_bindgen]
pub enum Group {
    Favorite = "favorite",
    Team = "team",
    Other = "other",
}

impl From<Availability> for CoreAvailability {
    fn from(value: Availability) -> Self {
        match value {
            Availability::Available => CoreAvailability::Available,
            Availability::Unavailable => CoreAvailability::Unavailable,
            Availability::DoNotDisturb => CoreAvailability::DoNotDisturb,
            Availability::Away => CoreAvailability::Away,
        }
    }
}

#[wasm_bindgen]
pub struct UserActivity(CoreUserActivity);

#[wasm_bindgen]
impl Contact {
    #[wasm_bindgen(getter)]
    pub fn jid(&self) -> BareJid {
        self.0.jid.clone().into()
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
    pub fn activity(&self) -> Option<UserActivity> {
        self.0
            .activity
            .as_ref()
            .map(|activity| UserActivity(activity.clone()))
    }

    #[wasm_bindgen(getter)]
    pub fn group(&self) -> Group {
        self.0.group.clone().into()
    }

    #[wasm_bindgen(getter, js_name = "isMe")]
    pub fn is_me(&self) -> bool {
        self.0.is_me
    }

    // pub avatar: Option<String>,
}

#[wasm_bindgen]
impl UserActivity {
    #[wasm_bindgen(constructor)]
    pub fn new(icon: &str, status: Option<String>) -> Self {
        UserActivity(CoreUserActivity {
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
        }
    }
}

impl From<CoreGroup> for Group {
    fn from(value: CoreGroup) -> Self {
        match value {
            CoreGroup::Favorite => Group::Favorite,
            CoreGroup::Team => Group::Team,
            CoreGroup::Other => Group::Other,
        }
    }
}
