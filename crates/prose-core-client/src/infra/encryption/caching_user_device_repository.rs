// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::app::deps::DynUserDeviceService;
use crate::domain::encryption::models::{Device, DeviceId};
use crate::domain::encryption::repos::UserDeviceRepository as UserDeviceRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::UserId;

pub struct CachingUserDeviceRepository {
    store: Store<PlatformDriver>,
    user_device_service: DynUserDeviceService,
    updated_devices: Mutex<HashSet<UserId>>,
}

impl CachingUserDeviceRepository {
    pub fn new(store: Store<PlatformDriver>, user_device_service: DynUserDeviceService) -> Self {
        Self {
            store,
            user_device_service,
            updated_devices: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserDeviceRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    device_id: DeviceId,
    label: Option<String>,
}

impl UserDeviceRecord {
    fn new(
        account: &AccountId,
        user_id: &UserId,
        device_id: DeviceId,
        label: Option<String>,
    ) -> Self {
        Self {
            id: format!("{}.{}.{}", account, user_id, device_id),
            account: account.clone(),
            user_id: user_id.clone(),
            device_id,
            label,
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const USER_ID: &str = "user_id";
    pub const DEVICE_ID: &str = "device_id";
}

define_entity!(UserDeviceRecord, "omemo_user_device",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    user_idx => { columns: [columns::ACCOUNT, columns::USER_ID], unique: false },
    device_idx => { columns: [columns::ACCOUNT, columns::USER_ID, columns::DEVICE_ID], unique: true }
);

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserDeviceRepositoryTrait for CachingUserDeviceRepository {
    async fn get_all(&self, account: &AccountId, user_id: &UserId) -> Result<Vec<Device>> {
        if self.updated_devices.lock().contains(user_id) {
            return self.fetch_devices(account, user_id).await;
        }

        let device_list = self.user_device_service.load_device_list(user_id).await?;
        self.set_all(account, user_id, device_list.devices.clone())
            .await?;
        Ok(device_list.devices)
    }

    async fn set_all(
        &self,
        account: &AccountId,
        user_id: &UserId,
        devices: Vec<Device>,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserDeviceRecord::collection())?;
        let idx = collection.index(&UserDeviceRecord::user_idx())?;

        let current_device_ids = idx
            .get_all_values::<UserDeviceRecord>(
                Query::Only((account, user_id)),
                Default::default(),
                None,
            )
            .await?
            .into_iter()
            .map(|record| record.device_id)
            .collect::<HashSet<_>>();

        let mut deleted_device_ids = current_device_ids.clone();

        for device in devices {
            deleted_device_ids.remove(&device.id);

            if current_device_ids.contains(&device.id) {
                continue;
            }

            collection.put_entity(&UserDeviceRecord::new(
                account,
                user_id,
                device.id,
                device.label,
            ))?;
        }

        let idx = collection.index(&UserDeviceRecord::device_idx())?;

        for device_id in deleted_device_ids {
            idx.delete(&(account, user_id, device_id)).await?;
        }

        tx.commit().await?;
        self.updated_devices.lock().insert(user_id.clone());

        Ok(())
    }

    async fn reset_before_reconnect(&self, _account: &AccountId) -> Result<()> {
        self.updated_devices.lock().clear();
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        self.updated_devices.lock().clear();

        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserDeviceRecord::collection())?;
        collection
            .delete_all_in_index(&UserDeviceRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

impl CachingUserDeviceRepository {
    async fn fetch_devices(&self, account: &AccountId, user_id: &UserId) -> Result<Vec<Device>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserDeviceRecord::collection())?;
        let idx = collection.index(&UserDeviceRecord::user_idx())?;

        let devices = idx
            .get_all_values::<UserDeviceRecord>(
                Query::Only((account, user_id)),
                Default::default(),
                None,
            )
            .await?
            .into_iter()
            .map(|record| Device {
                id: record.device_id,
                label: record.label,
            })
            .collect::<Vec<_>>();

        Ok(devices)
    }
}
