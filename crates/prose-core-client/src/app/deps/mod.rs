// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) use app_context::AppContext;

#[cfg(feature = "test")]
pub use app_dependencies::*;
#[cfg(not(feature = "test"))]
pub(crate) use app_dependencies::*;

mod app_context;
mod app_dependencies;
