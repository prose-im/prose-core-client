// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MamVersion {
    Mam0,
    Mam1,
    Mam2,
    Mam2Extended,
}
