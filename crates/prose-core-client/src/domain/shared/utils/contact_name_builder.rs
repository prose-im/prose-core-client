// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::app::dtos::UserProfile;
use crate::domain::contacts::models::Contact;
use crate::util::jid_ext::BareJidExt;
use std::ops::Deref;

pub(crate) fn build_contact_name(contact: &Contact, profile: &UserProfile) -> String {
    concatenate_names(&profile.first_name, &profile.last_name)
        .or_else(|| profile.nickname.clone())
        .or_else(|| contact.name.clone())
        .unwrap_or_else(|| contact.jid.to_display_name())
}

pub(crate) fn concatenate_names(
    first_name: &Option<String>,
    last_name: &Option<String>,
) -> Option<String> {
    let parts = first_name
        .iter()
        .chain(last_name.iter())
        .map(|s| s.deref())
        .collect::<Vec<_>>();

    (!parts.is_empty())
        .then_some(parts)
        .map(|parts| parts.join(" "))
}
