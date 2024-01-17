// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::Element;

pub use constant_time_provider::ConstantTimeProvider;
pub use message_builder::MessageBuilder;
pub use mock_app_dependencies::{
    MockAppDependencies, MockRoomFactoryDependencies, MockRoomsDomainServiceDependencies,
    MockSidebarDomainServiceDependencies,
};
use prose_xmpp::Client;
pub use room_internals::DisconnectedState;

use crate::app::event_handlers::ServerEvent;
use crate::parse_xmpp_event;

mod bookmark;
mod constant_time_provider;
mod message_builder;
mod mock_app_dependencies;
mod room_internals;
mod room_metadata;

pub mod mock_data {
    pub use super::mock_app_dependencies::{
        mock_account_jid as account_jid, mock_muc_service as muc_service,
        mock_reference_date as reference_date,
    };
}

#[macro_export]
macro_rules! room_id {
    ($jid:expr) => {
        RoomId::from($jid.parse::<jid::BareJid>().unwrap())
    };
}

#[macro_export]
macro_rules! user_id {
    ($jid:expr) => {
        UserId::from($jid.parse::<jid::BareJid>().unwrap())
    };
}

#[macro_export]
macro_rules! sender_id {
    ($jid:expr) => {
        SenderId::from($jid.parse::<jid::Jid>().unwrap())
    };
}

#[macro_export]
macro_rules! user_resource_id {
    ($jid:expr) => {
        UserResourceId::from($jid.parse::<jid::FullJid>().unwrap())
    };
}

#[macro_export]
macro_rules! occupant_id {
    ($jid:expr) => {
        OccupantId::from($jid.parse::<jid::FullJid>().unwrap())
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn parse_xml(xml: &str) -> Result<Vec<ServerEvent>> {
    use prose_xmpp::test::ClientTestAdditions;

    let client = Client::connected_client().await?;

    client
        .connection
        .receive_stanza(xml.trim().parse::<Element>()?)
        .await;

    let parsed_events = client
        .sent_events()
        .into_iter()
        .map(|e| parse_xmpp_event(e))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(parsed_events.into_iter().flatten().collect())
}
