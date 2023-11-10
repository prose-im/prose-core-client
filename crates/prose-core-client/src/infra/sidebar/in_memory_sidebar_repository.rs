// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use jid::BareJid;
use parking_lot::RwLock;

use crate::domain::sidebar::models::SidebarItem;
use crate::domain::sidebar::repos::SidebarRepository;

pub struct InMemorySidebarRepository {
    sidebar_items: RwLock<HashMap<BareJid, SidebarItem>>,
}

impl InMemorySidebarRepository {
    pub fn new() -> Self {
        Self {
            sidebar_items: Default::default(),
        }
    }
}

impl SidebarRepository for InMemorySidebarRepository {
    fn set_all(&self, items: Vec<SidebarItem>) {
        *self.sidebar_items.write() = items
            .into_iter()
            .map(|item| (item.jid.clone(), item))
            .collect()
    }

    fn get_all(&self) -> Vec<SidebarItem> {
        self.sidebar_items.read().values().cloned().collect()
    }

    fn get(&self, jid: &BareJid) -> Option<SidebarItem> {
        self.sidebar_items.read().get(jid).cloned()
    }

    fn put(&self, item: &SidebarItem) {
        self.sidebar_items
            .write()
            .insert(item.jid.clone(), item.clone());
    }

    fn delete(&self, item: &BareJid) {
        self.sidebar_items.write().remove(item);
    }

    fn clear_cache(&self) {
        self.sidebar_items.write().clear();
    }
}
