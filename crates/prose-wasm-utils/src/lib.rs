// prose-wasm-utils/prose-wasm-utils
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use future_ext::ProseFutureExt;
pub use receiver_stream::ReceiverStream;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
pub use stream_ext::ProseStreamExt;

mod future_ext;
mod receiver_stream;
mod stream_ext;

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

pub fn spawn<T>(future: T) -> ()
where
    T: Future + SendUnlessWasm + 'static,
    T::Output: SendUnlessWasm,
{
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        future.await;
    });
    #[cfg(all(not(target_arch = "wasm32")))]
    tokio::spawn(future);
}

pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(
        duration.as_millis().try_into().expect("Invalid duration"),
    )
    .await;

    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;
}
