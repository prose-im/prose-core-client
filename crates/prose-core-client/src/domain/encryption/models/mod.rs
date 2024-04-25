// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use device::{Device, DeviceList};
pub use device_bundle::{DeviceBundle, PreKeyBundle};
pub use device_id::DeviceId;
pub use device_info::{DeviceInfo, DeviceTrust};
pub use keys::*;
pub use local_device::LocalDevice;
pub use local_encryption_bundle::LocalEncryptionBundle;
pub use session::{Session, Trust};

mod device;
mod device_bundle;
mod device_id;
mod device_info;
mod keys;
mod local_device;
mod local_encryption_bundle;
mod session;
