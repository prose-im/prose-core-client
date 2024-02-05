// prose-wasm-utils/prose-wasm-utils
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use futures::{Stream, StreamExt};
use tokio::sync::mpsc::channel;

use crate::{spawn, ReceiverStream, SendUnlessWasm};

pub trait ProseStreamExt: Stream {
    fn throttled(self, interval: Duration) -> impl Stream<Item = Vec<Self::Item>>;
}

impl<T: Stream + SendUnlessWasm + 'static> ProseStreamExt for T
where
    T::Item: SendUnlessWasm,
{
    fn throttled(self, interval: Duration) -> impl Stream<Item = Vec<T::Item>> {
        let (tx, rx) = channel(1);

        spawn(async move {
            let mut interval = interval_stream(interval);
            let mut buf = Vec::new();
            let mut stream = Box::pin(self);

            _ = interval.next().await;

            loop {
                tokio::select! {
                    event = stream.next() => {
                        if let Some(event) = event {
                            buf.push(event);
                        } else {
                            if !buf.is_empty() {
                               _ = tx.send(buf).await;
                            }
                            break;
                        }
                    }
                    _ = interval.next() => {
                        if !buf.is_empty() {
                            let _ = tx.send(buf).await;
                            buf = Vec::new();
                        }
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
}

fn interval_stream(interval: Duration) -> impl Stream<Item = ()> + SendUnlessWasm {
    #[cfg(all(not(target_arch = "wasm32")))]
    return tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(interval))
        .map(|_| ());

    #[cfg(target_arch = "wasm32")]
    return gloo_timers::future::IntervalStream::new(interval.subsec_millis());
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use tokio::time::interval;
    use tokio_stream::wrappers::IntervalStream;

    use super::*;

    #[tokio::test]
    async fn test_throttle() {
        let stream = IntervalStream::new(interval(Duration::from_millis(15))).map(|_| 0);

        let delay = Duration::from_millis(100);
        let throttled_stream = stream.throttled(delay);
        let counter = Arc::new(AtomicU32::new(0));

        {
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut stream = pin!(throttled_stream);
                while let Some(events) = stream.next().await {
                    counter.fetch_add(events.len() as u32, Ordering::SeqCst);
                }
            });
        }

        assert_eq!(counter.load(Ordering::SeqCst), 0);

        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 7);

        tokio::time::sleep(Duration::from_millis(120)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 14);
    }
}
