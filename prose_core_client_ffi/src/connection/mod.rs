// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod libstrophe_connection;
mod xmpp_connection;

pub(crate) use libstrophe_connection::LibstropheConnection;
pub use xmpp_connection::{
    ConnectionEvent, ConnectionHandler, StanzaHandler, XMPPConnection, XMPPSender,
};
