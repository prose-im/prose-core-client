// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use parking_lot::Mutex;
use rand::Rng;

use crate::domain::encryption::models::DeviceId;

pub trait UserDeviceIdProvider: Send + Sync {
    fn new_id(&self) -> DeviceId;
}

#[derive(Default)]
pub struct RandUserDeviceIdProvider {}

impl UserDeviceIdProvider for RandUserDeviceIdProvider {
    fn new_id(&self) -> DeviceId {
        DeviceId::from(rand::thread_rng().gen_range(1..2u32.pow(31)))
    }
}

pub struct IncrementingUserDeviceIdProvider {
    last_id: Mutex<u32>,
}

impl IncrementingUserDeviceIdProvider {
    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        IncrementingUserDeviceIdProvider {
            last_id: Mutex::new(value),
        }
    }
}

impl UserDeviceIdProvider for IncrementingUserDeviceIdProvider {
    fn new_id(&self) -> DeviceId {
        let mut last_id = self.last_id.lock();
        let id = DeviceId::from(*last_id);
        *last_id += 1;
        id
    }
}
