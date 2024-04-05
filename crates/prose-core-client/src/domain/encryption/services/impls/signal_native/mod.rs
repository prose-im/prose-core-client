// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use signal_service::SignalServiceHandle;

pub(self) use signal_repo_wrapper::SignalRepoWrapper;

mod signal_compat;
mod signal_repo_wrapper;
mod signal_service;
