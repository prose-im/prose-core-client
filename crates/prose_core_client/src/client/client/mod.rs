pub use client::Client;
pub(crate) use client::ClientError;
pub use client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedDataCache};

mod client;
mod client_builder;
mod client_contacts;
mod client_conversation;
mod client_user;
