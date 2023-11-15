// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use parking_lot::RwLock;

use crate::domain::shared::models::RoomJid;
use crate::domain::sidebar::models::SidebarItem;
use crate::domain::sidebar::repos::{SidebarReadOnlyRepository, SidebarRepository};

pub struct InMemorySidebarRepository {
    sidebar_items: RwLock<HashMap<RoomJid, SidebarItem>>,
}

impl InMemorySidebarRepository {
    pub fn new() -> Self {
        Self {
            sidebar_items: Default::default(),
        }
    }
}

impl SidebarReadOnlyRepository for InMemorySidebarRepository {
    fn get(&self, jid: &RoomJid) -> Option<SidebarItem> {
        self.sidebar_items.read().get(jid).cloned()
    }

    fn get_all(&self) -> Vec<SidebarItem> {
        let mut items = self
            .sidebar_items
            .read()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        items
    }
}

impl SidebarRepository for InMemorySidebarRepository {
    fn put(&self, item: &SidebarItem) {
        self.sidebar_items
            .write()
            .insert(item.jid.clone(), item.clone());
    }

    fn delete(&self, item: &RoomJid) {
        self.sidebar_items.write().remove(item);
    }

    fn delete_all(&self) {
        self.sidebar_items.write().clear()
    }
}
