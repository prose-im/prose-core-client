// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Waker};

use anyhow::Result;
use minidom::Element;
use parking_lot::Mutex;
use tracing::error;
use xmpp_parsers::iq::IqType;

use crate::util::module_future_state::{ModuleFuturePoll, ModuleFutureState};
use crate::util::request_error::RequestError;
use crate::util::XMPPElement;

pub(crate) enum ElementReducerPoll {
    Pending,
    Ready,
}

type ElementReducer<T> =
    Box<dyn Fn(&mut T, &XMPPElement) -> Result<ElementReducerPoll, RequestError> + Send>;
type ResultTransformer<T, U> = fn(T) -> U;

pub(crate) struct RequestFuture<T: Send, U> {
    pub(crate) state: Arc<Mutex<ReducerFutureState<T, U>>>,
}

pub(crate) struct IQReducerState {
    request_id: String,
    element: Option<Element>,
}

impl RequestFuture<IQReducerState, Option<Element>> {
    pub fn new_iq_request(id: impl Into<String>) -> Self {
        let id = id.into();
        RequestFuture::new(
            id.clone(),
            IQReducerState {
                request_id: id,
                element: None,
            },
            |state, element| {
                let XMPPElement::IQ(iq) = element else {
                    return Ok(ElementReducerPoll::Pending);
                };

                if iq.id != state.request_id {
                    return Ok(ElementReducerPoll::Pending);
                }

                match &iq.payload {
                    IqType::Result(payload) => {
                        state.element = payload.clone();
                        Ok(ElementReducerPoll::Ready)
                    }
                    IqType::Error(err) => Err(RequestError::XMPP { err: err.clone() }),
                    IqType::Get(_) | IqType::Set(_) => Err(RequestError::UnexpectedResponse),
                }
            },
            |state| state.element,
        )
    }
}

impl<T: Send, U> RequestFuture<T, U> {
    pub fn new<R>(
        identifier: impl Into<String>,
        initial_value: T,
        reducer: R,
        transformer: ResultTransformer<T, U>,
    ) -> Self
    where
        R: Fn(&mut T, &XMPPElement) -> Result<ElementReducerPoll, RequestError> + Send + 'static,
    {
        RequestFuture {
            state: Arc::new(Mutex::new(ReducerFutureState {
                identifier: identifier.into(),
                reducer: Box::new(reducer),
                transformer,
                value: Some(initial_value),
                result: None,
                waker: None,
            })),
        }
    }

    pub fn failed(err: RequestError) -> Self {
        RequestFuture {
            state: Arc::new(Mutex::new(ReducerFutureState {
                identifier: "".to_string(),
                reducer: Box::new(|_, _| unreachable!()),
                transformer: |_| unreachable!(),
                value: None,
                result: Some(Err(err)),
                waker: None,
            })),
        }
    }
}

pub(crate) struct ReducerFutureState<T, U> {
    identifier: String,
    reducer: ElementReducer<T>,
    transformer: ResultTransformer<T, U>,
    value: Option<T>,
    result: Option<Result<(), RequestError>>,
    waker: Option<Waker>,
}

impl<T: Send, U> ModuleFutureState for ReducerFutureState<T, U> {
    fn handle_element(&mut self, element: &XMPPElement) -> ModuleFuturePoll {
        if self.result.is_some() {
            return ModuleFuturePoll::Ready(self.waker.take());
        }

        let mut value = self
            .value
            .take()
            .expect("Promise has been fulfilled already");
        let result = (self.reducer)(&mut value, element);
        self.value.replace(value);

        match result {
            Err(err) => {
                self.result = Some(Err(err));
                ModuleFuturePoll::Ready(self.waker.take())
            }
            Ok(ElementReducerPoll::Ready) => {
                self.result = Some(Ok(()));
                ModuleFuturePoll::Ready(self.waker.take())
            }
            Ok(ElementReducerPoll::Pending) => ModuleFuturePoll::Pending,
        }
    }

    fn fail_with_timeout(&mut self) -> Option<Waker> {
        error!("Request with id '{}' timed out.", self.identifier);
        self.result = Some(Err(RequestError::TimedOut));
        self.waker.take()
    }
}

impl<T: Send, U> Future for RequestFuture<T, U> {
    type Output = Result<U, RequestError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock();

        let Some(result) = state.result.take() else {
            state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        };

        match result {
            Ok(_) => {
                let value = (state.transformer)(
                    state
                        .value
                        .take()
                        .expect("Promise has been fulfilled already"),
                );
                Poll::Ready(Ok(value))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
