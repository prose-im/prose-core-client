pub use client::Client;
pub(crate) use client::ClientError;
pub(crate) use client_context::{ClientContext, XMPPClient};
pub use client_delegate::{ClientDelegate, ClientEvent};
pub(crate) use module_delegate::ModuleDelegate;

mod client;
mod client_context;
mod client_delegate;
mod module_delegate;
