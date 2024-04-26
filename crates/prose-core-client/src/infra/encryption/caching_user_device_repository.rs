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
use crate::dtos::UserId;
use crate::infra::encryption::{UserDeviceKey, UserDeviceKeyRef};

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
    pub id: UserDeviceKey,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub label: Option<String>,
}

impl UserDeviceRecord {
    fn user_id_idx() -> &'static str {
        "user_id"
    }
}

impl Entity for UserDeviceRecord {
    type ID = UserDeviceKey;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    fn collection() -> &'static str {
        "omemo_user_device"
    }

    fn indexes() -> Vec<IndexSpec> {
        vec![IndexSpec::builder(Self::user_id_idx()).build()]
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserDeviceRepositoryTrait for CachingUserDeviceRepository {
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<Device>> {
        if self.updated_devices.lock().contains(user_id) {
            return self.fetch_devices(user_id).await;
        }

        let device_list = self.user_device_service.load_device_list(user_id).await?;
        self.set_all(user_id, device_list.devices.clone()).await?;
        Ok(device_list.devices)
    }

    async fn set_all(&self, user_id: &UserId, devices: Vec<Device>) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserDeviceRecord::collection())?;
        let idx = collection.index(UserDeviceRecord::user_id_idx())?;

        let current_device_ids = idx
            .get_all_values::<UserDeviceRecord>(
                Query::Only(user_id.clone()),
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

            collection.put_entity(&UserDeviceRecord {
                id: UserDeviceKey::new(user_id, &device.id),
                user_id: user_id.clone(),
                device_id: device.id,
                label: device.label,
            })?;
        }

        for device_id in deleted_device_ids {
            collection.delete(&UserDeviceKeyRef::new(user_id, &device_id))?;
        }

        tx.commit().await?;
        self.updated_devices.lock().insert(user_id.clone());

        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        self.updated_devices.lock().clear();

        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        tx.truncate_collections(&[UserDeviceRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}

impl CachingUserDeviceRepository {
    async fn fetch_devices(&self, user_id: &UserId) -> Result<Vec<Device>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserDeviceRecord::collection())?;
        let idx = collection.index(UserDeviceRecord::user_id_idx())?;

        let devices = idx
            .get_all_values::<UserDeviceRecord>(
                Query::Only(user_id.clone()),
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
