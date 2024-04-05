// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::encryption::models::{Device, DeviceId};
use crate::domain::encryption::repos::UserDeviceRepository as UserDeviceRepositoryTrait;
use crate::dtos::UserId;
use crate::infra::encryption::UserDeviceKey;

pub struct UserDeviceRepository {
    store: Store<PlatformDriver>,
}

impl UserDeviceRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
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
impl UserDeviceRepositoryTrait for UserDeviceRepository {
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<Device>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserDeviceRecord::collection())?;
        let idx = collection.index(UserDeviceRecord::user_id_idx())?;

        Ok(idx
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
            .collect())
    }

    async fn put_all(&self, user_id: &UserId, devices: Vec<Device>) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserDeviceRecord::collection())?;

        for device in devices {
            collection.put_entity(&UserDeviceRecord {
                id: UserDeviceKey::new(user_id, &device.id),
                user_id: user_id.clone(),
                device_id: device.id,
                label: device.label,
            })?;
        }

        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserDeviceRecord::collection()])
            .await?;
        tx.truncate_collections(&[UserDeviceRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}
