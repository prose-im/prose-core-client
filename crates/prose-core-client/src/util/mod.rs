// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

pub(crate) use form_config::FormConfig;
pub(crate) use presence_map::PresenceMap;
pub(crate) use string_ext::StringExt;

pub(crate) mod form_config;
mod presence_map;
mod string_ext;

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
