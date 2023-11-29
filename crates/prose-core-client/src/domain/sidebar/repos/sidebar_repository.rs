// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::RoomId;
use crate::domain::sidebar::models::SidebarItem;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait SidebarReadOnlyRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, jid: &RoomId) -> Option<SidebarItem>;
    fn get_all(&self) -> Vec<SidebarItem>;
}

pub trait SidebarRepository: SidebarReadOnlyRepository {
    fn put(&self, item: &SidebarItem);

    fn delete(&self, item: &RoomId);
    fn delete_all(&self);
}

#[cfg(feature = "test")]
mockall::mock! {
    pub SidebarReadWriteRepository {}

    impl SidebarReadOnlyRepository for SidebarReadWriteRepository {
        fn get(&self, jid: &RoomId) -> Option<SidebarItem>;
        fn get_all(&self) -> Vec<SidebarItem>;
    }

    impl SidebarRepository for SidebarReadWriteRepository {
        fn put(&self, item: &SidebarItem);
        fn delete(&self, item: &RoomId);
        fn delete_all(&self);
    }
}
