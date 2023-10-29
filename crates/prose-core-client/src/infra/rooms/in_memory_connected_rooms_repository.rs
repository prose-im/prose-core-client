// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use parking_lot::RwLock;

use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::repos::{ConnectedRoomsRepository, RoomAlreadyExistsError};

pub struct InMemoryConnectedRoomsRepository {
    rooms: RwLock<HashMap<BareJid, Arc<RoomInternals>>>,
}

impl InMemoryConnectedRoomsRepository {
    pub fn new() -> Self {
        InMemoryConnectedRoomsRepository {
            rooms: Default::default(),
        }
    }
}

impl ConnectedRoomsRepository for InMemoryConnectedRoomsRepository {
    fn get(&self, room_jid: &BareJid) -> Option<Arc<RoomInternals>> {
        self.rooms.read().get(room_jid).cloned()
    }

    fn get_all(&self) -> Vec<Arc<RoomInternals>> {
        self.rooms.read().values().cloned().collect()
    }

    fn set(&self, room: RoomInternals) -> Result<(), RoomAlreadyExistsError> {
        let mut rooms = self.rooms.write();

        if rooms.contains_key(&room.info.jid) {
            return Err(RoomAlreadyExistsError);
        }

        rooms.insert(room.info.jid.clone(), Arc::new(room));
        Ok(())
    }

    fn replace(&self, rooms: Vec<RoomInternals>) {
        *self.rooms.write() = rooms
            .into_iter()
            .map(|room| (room.info.jid.clone(), Arc::new(room)))
            .collect();
    }

    fn update(
        &self,
        room_jid: &BareJid,
        block: Box<dyn FnOnce(Arc<RoomInternals>) -> RoomInternals + Send>,
    ) -> Option<Arc<RoomInternals>> {
        let mut rooms = self.rooms.write();
        let Some(room) = rooms.remove(&room_jid) else {
            return None;
        };
        let modified_room = Arc::new(block(room));
        rooms.insert(room_jid.clone(), modified_room.clone());
        Some(modified_room)
    }

    fn delete(&self, room_jids: &[&BareJid]) {
        let jids_to_delete = room_jids.iter().map(|jid| *jid).collect::<HashSet<_>>();
        self.rooms
            .write()
            .retain(|room_jid, _| !jids_to_delete.contains(room_jid));
    }
}
