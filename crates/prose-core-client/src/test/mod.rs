// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use constant_time_provider::ConstantTimeProvider;
pub use message_builder::MessageBuilder;
pub use mock_app_dependencies::MockAppDependencies;

mod constant_time_provider;
mod message_builder;
mod mock_app_dependencies;

pub mod mock_data {
    pub use super::mock_app_dependencies::{
        mock_account_jid as account_jid, mock_muc_service as muc_service,
        mock_reference_date as reference_date,
    };
}
