// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod dtos;
pub mod services;

#[cfg(feature = "test")]
pub mod deps;
#[cfg(not(feature = "test"))]
pub(crate) mod deps;

#[cfg(feature = "test")]
pub mod event_handlers;
#[cfg(not(feature = "test"))]
pub(crate) mod event_handlers;
