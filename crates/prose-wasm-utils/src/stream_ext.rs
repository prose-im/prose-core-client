// prose-wasm-utils/prose-wasm-utils
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;

use crate::{spawn, ReceiverStream, SendUnlessWasm};
use futures::{Stream, StreamExt};
use tokio::sync::mpsc::channel;

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Instant};

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

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
            let mut buf = Vec::new();
            let mut stream = Box::pin(self);
            let mut last_emit = Instant::now();

            enum State {
                Idle,
                Buffering,
            }

            let mut state = State::Idle;

            loop {
                match state {
                    State::Idle => {
                        let Some(event) = stream.next().await else {
                            break;
                        };
                        buf.push(event);
                        state = State::Buffering;
                        last_emit = Instant::now();
                    }
                    State::Buffering => {
                        let deadline = last_emit + interval;

                        tokio::select! {
                            event = stream.next() => {
                                if let Some(event) = event {
                                    buf.push(event);
                                } else {
                                    if !buf.is_empty() {
                                        let _ = tx.send(buf).await;
                                    }
                                    break;
                                }
                            }
                            _ = sleep(deadline.saturating_duration_since(Instant::now())) => {
                                if !buf.is_empty() {
                                    let _ = tx.send(buf).await;
                                    buf = Vec::new();
                                }
                                state = State::Idle;
                            }
                        }
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
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
