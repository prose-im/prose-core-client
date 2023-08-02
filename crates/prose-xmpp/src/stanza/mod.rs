pub use xmpp_parsers::presence;

pub use last_activity::LastActivityRequest;
pub use message::Message;
pub use pubsub::PubSubMessage;
pub use user_activity::UserActivity;
pub use vcard::VCard4;

pub mod avatar;
pub mod last_activity;
pub mod message;
pub mod ns;
pub mod pubsub;
pub mod user_activity;
pub mod vcard;
