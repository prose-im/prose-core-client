use crate::types::{IntoJSStringArray, StringArray};
use prose_core_client::types::{
    Availability as ProseAvailability, Contact as ProseContact, UserActivity as ProseUserActivity,
};
use wasm_bindgen::prelude::*;

use super::BareJid;

#[wasm_bindgen]
pub struct Contact(ProseContact);

impl From<ProseContact> for Contact {
    fn from(value: ProseContact) -> Self {
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

impl From<Availability> for ProseAvailability {
    fn from(value: Availability) -> Self {
        match value {
            Availability::Available => ProseAvailability::Available,
            Availability::Unavailable => ProseAvailability::Unavailable,
            Availability::DoNotDisturb => ProseAvailability::DoNotDisturb,
            Availability::Away => ProseAvailability::Away,
        }
    }
}

#[wasm_bindgen]
pub struct UserActivity(ProseUserActivity);

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
    pub fn groups(&self) -> StringArray {
        self.0.groups.iter().collect_into_js_string_array()
    }

    // pub avatar: Option<String>,
}

#[wasm_bindgen]
impl UserActivity {
    #[wasm_bindgen(constructor)]
    pub fn new(icon: &str, status: Option<String>) -> Self {
        UserActivity(ProseUserActivity {
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

impl From<ProseAvailability> for Availability {
    fn from(value: ProseAvailability) -> Self {
        match value {
            ProseAvailability::Available => Availability::Available,
            ProseAvailability::Unavailable => Availability::Unavailable,
            ProseAvailability::DoNotDisturb => Availability::DoNotDisturb,
            ProseAvailability::Away => Availability::Away,
        }
    }
}
