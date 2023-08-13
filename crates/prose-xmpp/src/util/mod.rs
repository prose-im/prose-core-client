// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::pin::Pin;

pub use element_ext::ElementExt;
pub(crate) use module_future_state::{ModuleFuturePoll, ModuleFutureState};
pub use request_error::RequestError;
pub(crate) use request_future::{ElementReducerPoll, RequestFuture};
pub use xmpp_element::XMPPElement;

mod element_ext;
pub(crate) mod id_string_macro;
mod module_future_state;
mod request_error;
mod request_future;
mod xmpp_element;

#[cfg(not(target_arch = "wasm32"))]
pub trait SendUnlessWasm: Send {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> SendUnlessWasm for T {}

#[cfg(target_arch = "wasm32")]
pub trait SendUnlessWasm {}

#[cfg(target_arch = "wasm32")]
impl<T> SendUnlessWasm for T {}

#[cfg(not(target_arch = "wasm32"))]
pub trait SyncUnlessWasm: Sync {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync> SyncUnlessWasm for T {}

#[cfg(target_arch = "wasm32")]
pub trait SyncUnlessWasm {}

#[cfg(target_arch = "wasm32")]
impl<T> SyncUnlessWasm for T {}

#[cfg(target_arch = "wasm32")]
pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;
#[cfg(not(target_arch = "wasm32"))]
pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
