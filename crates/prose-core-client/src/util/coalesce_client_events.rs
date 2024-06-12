// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;

use crate::dtos::RoomEnvelope;
use crate::{ClientEvent, ClientRoomEventType};

pub fn coalesce_client_events(events: &mut Vec<ClientEvent>) {
    events.sort_by(compare_client_events);
    events.dedup_by(|lhs, rhs| match (lhs, rhs) {
        (
            ClientEvent::ConnectionStatusChanged { .. },
            ClientEvent::ConnectionStatusChanged { .. },
        ) => true,
        (ClientEvent::SidebarChanged, ClientEvent::SidebarChanged) => true,
        (
            ClientEvent::ContactChanged { ids: ids_a },
            ClientEvent::ContactChanged { ids: ids_b },
        ) => {
            ids_b.extend(ids_a.drain(..));
            true
        }
        (ClientEvent::ContactListChanged, ClientEvent::ContactListChanged) => true,
        (ClientEvent::PresenceSubRequestsChanged, ClientEvent::PresenceSubRequestsChanged) => true,
        (ClientEvent::BlockListChanged, ClientEvent::BlockListChanged) => true,
        (ClientEvent::AvatarChanged { ids: ids_a }, ClientEvent::AvatarChanged { ids: ids_b }) => {
            ids_b.extend(ids_a.drain(..));
            true
        }
        (ClientEvent::AccountInfoChanged, ClientEvent::AccountInfoChanged) => true,
        (
            ClientEvent::RoomChanged {
                room: room_a,
                r#type: type_a,
            },
            ClientEvent::RoomChanged {
                room: room_b,
                r#type: type_b,
            },
        ) => should_dedup_room_events(room_a, room_b, type_a, type_b),

        (ClientEvent::ConnectionStatusChanged { .. }, _) => false,
        (ClientEvent::SidebarChanged, _) => false,
        (ClientEvent::ContactChanged { .. }, _) => false,
        (ClientEvent::ContactListChanged, _) => false,
        (ClientEvent::PresenceSubRequestsChanged, _) => false,
        (ClientEvent::BlockListChanged, _) => false,
        (ClientEvent::AvatarChanged { .. }, _) => false,
        (ClientEvent::AccountInfoChanged, _) => false,
        (ClientEvent::RoomChanged { .. }, _) => false,
    });
}

fn should_dedup_room_events(
    room_a: &mut RoomEnvelope,
    room_b: &mut RoomEnvelope,
    event_a: &mut ClientRoomEventType,
    event_b: &mut ClientRoomEventType,
) -> bool {
    if room_a.to_generic_room().jid() != room_b.to_generic_room().jid() {
        return false;
    }

    match (event_a, event_b) {
        (
            ClientRoomEventType::MessagesAppended { message_ids: ids_a },
            ClientRoomEventType::MessagesAppended { message_ids: ids_b },
        ) => {
            ids_b.extend(ids_a.drain(..));
            true
        }
        (
            ClientRoomEventType::MessagesUpdated { message_ids: ids_a },
            ClientRoomEventType::MessagesUpdated { message_ids: ids_b },
        ) => {
            ids_b.extend(ids_a.drain(..));
            true
        }
        (
            ClientRoomEventType::MessagesDeleted { message_ids: ids_a },
            ClientRoomEventType::MessagesDeleted { message_ids: ids_b },
        ) => {
            ids_b.extend(ids_a.drain(..));
            true
        }
        (ClientRoomEventType::MessagesNeedReload, ClientRoomEventType::MessagesNeedReload) => true,
        (ClientRoomEventType::AttributesChanged, ClientRoomEventType::AttributesChanged) => true,
        (ClientRoomEventType::ParticipantsChanged, ClientRoomEventType::ParticipantsChanged) => {
            true
        }
        (
            ClientRoomEventType::ComposingUsersChanged,
            ClientRoomEventType::ComposingUsersChanged,
        ) => true,

        (ClientRoomEventType::MessagesAppended { .. }, _) => false,
        (ClientRoomEventType::MessagesUpdated { .. }, _) => false,
        (ClientRoomEventType::MessagesDeleted { .. }, _) => false,
        (ClientRoomEventType::MessagesNeedReload, _) => false,
        (ClientRoomEventType::AttributesChanged, _) => false,
        (ClientRoomEventType::ParticipantsChanged, _) => false,
        (ClientRoomEventType::ComposingUsersChanged, _) => false,
    }
}

fn compare_client_events(event_a: &ClientEvent, event_b: &ClientEvent) -> Ordering {
    match (event_a, event_b) {
        (
            ClientEvent::RoomChanged {
                room: room_a,
                r#type: type_a,
            },
            ClientEvent::RoomChanged {
                room: room_b,
                r#type: type_b,
            },
        ) => {
            let room_a = room_a.to_generic_room();
            let room_b = room_b.to_generic_room();

            if room_a.jid() == room_b.jid() {
                order_key_for_room_event(&type_a).cmp(&order_key_for_room_event(&type_b))
            } else {
                room_a.jid().as_ref().cmp(room_b.jid().as_ref())
            }
        }
        _ => order_key_for_client_event(event_a).cmp(&order_key_for_client_event(event_b)),
    }
}

fn order_key_for_client_event(event: &ClientEvent) -> i32 {
    match event {
        ClientEvent::ConnectionStatusChanged { .. } => 0,
        ClientEvent::SidebarChanged => 1,
        ClientEvent::ContactChanged { .. } => 2,
        ClientEvent::ContactListChanged => 3,
        ClientEvent::PresenceSubRequestsChanged => 4,
        ClientEvent::BlockListChanged => 5,
        ClientEvent::AvatarChanged { .. } => 6,
        ClientEvent::AccountInfoChanged => 7,
        ClientEvent::RoomChanged { .. } => 8,
    }
}

fn order_key_for_room_event(event: &ClientRoomEventType) -> i32 {
    match event {
        ClientRoomEventType::MessagesAppended { .. } => 0,
        ClientRoomEventType::MessagesUpdated { .. } => 1,
        ClientRoomEventType::MessagesDeleted { .. } => 2,
        ClientRoomEventType::MessagesNeedReload => 3,
        ClientRoomEventType::AttributesChanged => 4,
        ClientRoomEventType::ParticipantsChanged => 5,
        ClientRoomEventType::ComposingUsersChanged => 6,
    }
}

#[cfg(test)]
mod tests {
    use crate::dtos::UserId;
    use crate::user_id;

    use super::*;

    #[test]
    fn test_coalesce_events() {
        let mut events = vec![
            ClientEvent::SidebarChanged,
            ClientEvent::SidebarChanged,
            ClientEvent::ContactListChanged,
            ClientEvent::AvatarChanged {
                ids: vec![user_id!("a@prose.org")],
            },
            ClientEvent::AccountInfoChanged,
            ClientEvent::SidebarChanged,
            ClientEvent::AvatarChanged {
                ids: vec![user_id!("b@prose.org")],
            },
        ];
        coalesce_client_events(&mut events);

        assert_eq!(
            events,
            vec![
                ClientEvent::SidebarChanged,
                ClientEvent::ContactListChanged,
                ClientEvent::AvatarChanged {
                    ids: vec![user_id!("a@prose.org"), user_id!("b@prose.org")],
                },
                ClientEvent::AccountInfoChanged,
            ]
        );
    }
}
