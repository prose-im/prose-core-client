// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use contact::Contact;

mod contact;

pub use crate::domain::{
    messaging::models::Message,
    rooms::models::Occupant,
    shared::models::Availability,
    user_profiles::models::{Address, UserProfile},
};
