// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use element_ext::{parse_bool, ElementBuilderExt, ElementExt};
pub(crate) use module_future_state::{ModuleFuturePoll, ModuleFutureState};
pub use pub_sub_items_ext::PubSubItemsExt;
pub use pub_sub_query::PubSubQuery;
pub use publish_options_ext::PublishOptionsExt;
pub use request_error::{ParseError, RequestError};
pub(crate) use request_future::{ElementReducerPoll, RequestFuture};
pub use xmpp_element::XMPPElement;

pub mod element_ext;
mod module_future_state;
mod pub_sub_items_ext;
mod pub_sub_query;
mod publish_options_ext;
mod request_error;
mod request_future;
mod xmpp_element;
