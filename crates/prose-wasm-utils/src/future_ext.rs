// prose-wasm-utils/prose-wasm-utils
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use futures::FutureExt;
use std::future::Future;

impl<T: ?Sized> ProseFutureExt for T where T: Future {}

pub trait ProseFutureExt: Future {
    #[cfg(target_arch = "wasm32")]
    fn prose_boxed<'a>(self) -> futures::future::LocalBoxFuture<'a, Self::Output>
    where
        Self: Sized + 'a,
    {
        self.boxed_local()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn prose_boxed<'a>(self) -> futures::future::BoxFuture<'a, Self::Output>
    where
        Self: Sized + Send + 'a,
    {
        self.boxed()
    }
}
