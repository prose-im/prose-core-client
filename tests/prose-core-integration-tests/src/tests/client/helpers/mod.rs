// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

pub use element_ext::ElementExt;
use prose_core_client::domain::encryption::repos::mocks::{
    MockEncryptionKeysRepository, MockSessionRepository,
};
use prose_core_client::domain::shared::models::AccountId;
use prose_core_client::dtos::{DeviceBundle, DeviceId};
use prose_core_client::infra::general::mocks::StepRngProvider;
use prose_core_client::{EncryptionService, SignalServiceHandle};
pub use test_client::TestClient;
pub use test_client_login::LoginStrategy;
pub use test_client_muc::{JoinRoomStrategy, StartDMStrategy};
pub(self) use test_message_queue::TestMessageQueue;

mod connector;
mod delegate;
mod element_ext;
mod id_provider;
mod test_client;
mod test_client_login;
mod test_client_muc;
mod test_client_omemo;
mod test_message_queue;

pub trait TestDeviceBundle {
    async fn test(account_id: impl Into<AccountId>, device_id: impl Into<DeviceId>)
        -> DeviceBundle;
}

impl TestDeviceBundle for DeviceBundle {
    async fn test(
        account_id: impl Into<AccountId>,
        device_id: impl Into<DeviceId>,
    ) -> DeviceBundle {
        let service = SignalServiceHandle::new(
            Arc::new(MockEncryptionKeysRepository::new()),
            Arc::new(MockSessionRepository::new()),
            Arc::new(StepRngProvider::default()),
        );
        service
            .generate_local_encryption_bundle(&account_id.into(), device_id.into())
            .await
            .unwrap()
            .into_device_bundle()
    }
}
