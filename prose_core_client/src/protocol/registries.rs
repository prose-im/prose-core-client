// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use super::namespaces;

// -- Constants --

pub const FEATURES: &'static [&str] = &[
    namespaces::DISCO_INFO,
    namespaces::DISCO_ITEMS,
    namespaces::NS_VERSION,
    namespaces::NS_LAST,
    namespaces::NS_URN_TIME,
    namespaces::NS_URN_PING,
];
