// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

/// Helper function to run futures in parallel in production or serially in tests
pub async fn join_all<I>(iter: I) -> Vec<<I::Item as Future>::Output>
where
    I: IntoIterator,
    I::Item: Future,
{
    #[cfg(feature = "test")]
    {
        // Run futures serially in tests
        let mut results = Vec::new();
        for future in iter.into_iter() {
            results.push(future.await);
        }
        results
    }
    #[cfg(not(feature = "test"))]
    {
        // Run futures in parallel in production
        futures::future::join_all(iter).await
    }
}
