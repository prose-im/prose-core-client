// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use xmpp_parsers::presence;

pub use conference_bookmark::ConferenceBookmark;
pub use last_activity::LastActivityRequest;
pub use message::Message;
pub use pubsub::PubSubMessage;
pub use user_activity::UserActivity;
pub use vcard::VCard4;

pub mod avatar;
pub mod conference_bookmark;
pub mod last_activity;
pub mod message;
pub mod muc;
pub mod ns;
pub mod pubsub;
pub mod user_activity;
pub mod vcard;
