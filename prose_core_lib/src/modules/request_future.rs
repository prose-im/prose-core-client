use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::stanza::{StanzaBase, IQ};

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("request error: {msg}")]
    Generic { msg: String },
    #[error("request timeout")]
    TimedOut,
    #[error("request error: Unexpected server response")]
    UnexpectedResponse,
}

impl<'a> TryFrom<&IQ<'a>> for RequestError {
    type Error = ();

    fn try_from(stanza: &IQ<'a>) -> Result<Self, Self::Error> {
        let Some(error) = stanza.child_by_name("error") else {
            return Err(())
        };
        Ok(RequestError::Generic {
            msg: error.to_string(),
        })
    }
}

pub(crate) struct RequestFuture {
    pub state: Arc<Mutex<RequestFutureState>>,
}

pub(crate) struct RequestFutureState {
    value: Option<Result<IQ<'static>, RequestError>>,
    waker: Option<Waker>,
}

impl RequestFuture {
    pub(crate) fn new() -> Self {
        RequestFuture {
            state: Arc::new(Mutex::new(RequestFutureState {
                value: None,
                waker: None,
            })),
        }
    }
}

impl RequestFutureState {
    pub(crate) fn fulfill(&mut self, value: IQ<'static>) {
        self.value = Some(Ok(value));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }

    pub(crate) fn fail(&mut self, error: RequestError) {
        self.value = Some(Err(error));
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

impl Future for RequestFuture {
    type Output = Result<IQ<'static>, RequestError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state_guard = self.state.lock().expect("Could not acquire mutex.");

        let Some(result) = state_guard.value.take() else {
            state_guard.waker = Some(cx.waker().clone());
            return Poll::Pending
        };
        return Poll::Ready(result);
    }
}
