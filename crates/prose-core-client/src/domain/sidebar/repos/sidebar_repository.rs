// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::sidebar::models::SidebarItem;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait SidebarRepository: SendUnlessWasm + SyncUnlessWasm {
    fn set_all(&self, items: Vec<SidebarItem>);
    fn get_all(&self) -> Vec<SidebarItem>;
    fn get(&self, jid: &BareJid) -> Option<SidebarItem>;
    fn put(&self, item: &SidebarItem);
    fn delete(&self, item: &BareJid);

    fn clear_cache(&self);
}
