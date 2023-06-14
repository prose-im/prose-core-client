use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use strum_macros::{Display, EnumString};

use crate::stanza::{Stanza, StanzaBase, IQ};

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("request error: {msg}")]
    Generic { msg: String },
    #[error("request timeout")]
    TimedOut,
    #[error("request error: Unexpected server response")]
    UnexpectedResponse,
    #[error("XMPP error: {err}")]
    XMPP { err: XMPPError },
}

#[derive(Debug)]
pub struct XMPPError {
    pub kind: Option<XMPPErrorKind>,
    pub text: Option<String>,
    pub status: Option<XMPPErrorStatus>,
}

impl Display for XMPPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Debug, PartialEq, Display, EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum XMPPErrorKind {
    Auth,
    Cancel,
    Modify,
    Wait,
}

#[derive(Debug, PartialEq, Display, EnumString, Clone)]
pub enum XMPPErrorStatus {
    #[strum(serialize = "bad-request")]
    BadRequest,
    #[strum(serialize = "conflict")]
    Conflict,
    #[strum(serialize = "feature-not-implemented")]
    FeatureNotImplemented,
    #[strum(serialize = "forbidden")]
    Forbidden,
    #[strum(serialize = "gone")]
    Gone,
    #[strum(serialize = "internal-server-error")]
    InternalServerError,
    #[strum(serialize = "item-not-found")]
    ItemNotFound,
    #[strum(serialize = "jid-malformed")]
    JidMalformed,
    #[strum(serialize = "not-acceptable")]
    NotAcceptable,
    #[strum(serialize = "not-allowed")]
    NotAllowed,
    #[strum(serialize = "not-authorized")]
    NotAuthorized,
    #[strum(serialize = "payment-required")]
    PaymentRequired,
    #[strum(serialize = "recipient-unavailable")]
    RecipientUnavailable,
    #[strum(serialize = "redirect")]
    Redirect,
    #[strum(serialize = "registration-required")]
    RegistrationRequired,
    #[strum(serialize = "remote-server-not-found")]
    RemoteServerNotFound,
    #[strum(serialize = "remote-server-timeout")]
    RemoteServerTimeout,
    #[strum(serialize = "resource-constraint")]
    ResourceConstraint,
    #[strum(serialize = "service-unavailable")]
    ServiceUnavailable,
    #[strum(serialize = "subscription-required")]
    SubscriptionRequired,
    #[strum(serialize = "undefined-condition")]
    UndefinedCondition,
    #[strum(serialize = "unexpected-request")]
    UnexpectedRequest,
}

impl<'a> TryFrom<&IQ<'a>> for RequestError {
    type Error = ();

    fn try_from(stanza: &IQ<'a>) -> Result<Self, Self::Error> {
        let Some(error) = stanza.child_by_name("error") else {
            return Err(())
        };

        if let Ok(xmpp_err) = XMPPError::try_from(&error) {
            return Ok(RequestError::XMPP { err: xmpp_err });
        }

        Ok(RequestError::Generic {
            msg: error.to_string(),
        })
    }
}

impl<'a> TryFrom<&Stanza<'a>> for XMPPError {
    type Error = ();

    fn try_from(stanza: &Stanza<'a>) -> Result<Self, Self::Error> {
        Ok(XMPPError {
            kind: stanza
                .attribute("type")
                .and_then(|attr| attr.parse::<XMPPErrorKind>().ok()),
            text: stanza.child_by_name("text").and_then(|s| s.text()),
            status: stanza
                .first_child()
                .and_then(|s| s.name().map(ToOwned::to_owned))
                .and_then(|s| s.parse::<XMPPErrorStatus>().ok()),
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
