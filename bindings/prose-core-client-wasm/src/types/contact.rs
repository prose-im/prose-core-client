use crate::types::{IntoJSStringArray, StringArray};
use wasm_bindgen::prelude::*;

use super::BareJid;

#[wasm_bindgen]
pub struct Contact(prose_domain::Contact);

impl From<prose_domain::Contact> for Contact {
    fn from(value: prose_domain::Contact) -> Self {
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
    pub fn status(&self) -> Option<String> {
        self.0.status.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn groups(&self) -> StringArray {
        self.0.groups.iter().collect_into_js_string_array()
    }

    // pub avatar: Option<String>,
}

impl From<prose_domain::Availability> for Availability {
    fn from(value: prose_domain::Availability) -> Self {
        match value {
            prose_domain::Availability::Available => Availability::Available,
            prose_domain::Availability::Unavailable => Availability::Unavailable,
            prose_domain::Availability::DoNotDisturb => Availability::DoNotDisturb,
            prose_domain::Availability::Away => Availability::Away,
        }
    }
}
