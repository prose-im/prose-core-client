// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use constant_time_provider::ConstantTimeProvider;
pub use message_builder::MessageBuilder;
pub use mock_app_dependencies::{
    MockAppDependencies, MockRoomFactoryDependencies, MockRoomsDomainServiceDependencies,
    MockSidebarDomainServiceDependencies,
};

mod bookmark;
mod constant_time_provider;
mod message_builder;
mod mock_app_dependencies;
mod room_internals;
mod room_metadata;
mod sidebar_item;

pub mod mock_data {
    pub use super::mock_app_dependencies::{
        mock_account_jid as account_jid, mock_muc_service as muc_service,
        mock_reference_date as reference_date,
    };
}

#[macro_export]
macro_rules! room {
    ($jid:expr) => {
        RoomJid::from($jid.parse::<jid::BareJid>().unwrap())
    };
}
