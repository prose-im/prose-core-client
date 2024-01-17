// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;

use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::repos::{
    ConnectedRoomsReadOnlyRepository, ConnectedRoomsRepository, RoomAlreadyExistsError,
};
use crate::domain::shared::models::RoomId;

pub struct InMemoryConnectedRoomsRepository {
    rooms: RwLock<HashMap<RoomId, Arc<RoomInternals>>>,
}

impl InMemoryConnectedRoomsRepository {
    pub fn new() -> Self {
        InMemoryConnectedRoomsRepository {
            rooms: Default::default(),
        }
    }
}

impl ConnectedRoomsReadOnlyRepository for InMemoryConnectedRoomsRepository {
    fn get(&self, room_id: &RoomId) -> Option<Arc<RoomInternals>> {
        self.rooms.read().get(room_id).cloned()
    }

    fn get_all(&self) -> Vec<Arc<RoomInternals>> {
        self.rooms.read().values().cloned().collect()
    }
}

impl ConnectedRoomsRepository for InMemoryConnectedRoomsRepository {
    fn set(&self, room: Arc<RoomInternals>) -> Result<(), RoomAlreadyExistsError> {
        let mut rooms = self.rooms.write();

        if rooms.contains_key(&room.room_id) {
            return Err(RoomAlreadyExistsError);
        }

        rooms.insert(room.room_id.clone(), room);
        Ok(())
    }

    fn set_or_replace(&self, room: Arc<RoomInternals>) -> Option<Arc<RoomInternals>> {
        let mut rooms = self.rooms.write();
        rooms.insert(room.room_id.clone(), room)
    }

    fn update(
        &self,
        room_id: &RoomId,
        block: Box<dyn FnOnce(Arc<RoomInternals>) -> RoomInternals + Send>,
    ) -> Option<Arc<RoomInternals>> {
        let mut rooms = self.rooms.write();
        let Some(room) = rooms.remove(&room_id) else {
            return None;
        };
        let modified_room = Arc::new(block(room));
        rooms.insert(room_id.clone(), modified_room.clone());
        Some(modified_room)
    }

    fn delete(&self, room_id: &RoomId) -> Option<Arc<RoomInternals>> {
        self.rooms.write().remove(room_id)
    }

    fn delete_all(&self) -> Vec<Arc<RoomInternals>> {
        let rooms = &mut *self.rooms.write();
        let deleted_map = mem::replace(rooms, HashMap::new());
        deleted_map.into_values().collect()
    }
}
