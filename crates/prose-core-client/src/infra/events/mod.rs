// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(not(feature = "test"))]
pub use coalescing_client_event_dispatcher::CoalescingClientEventDispatcher;
#[cfg(feature = "test")]
pub use immediate_client_event_dispatcher::ImmediateClientEventDispatcher;

#[cfg(not(feature = "test"))]
mod coalescing_client_event_dispatcher;
#[cfg(feature = "test")]
mod immediate_client_event_dispatcher;
