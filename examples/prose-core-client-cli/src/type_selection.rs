// prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::iter;
use std::path::{Path, PathBuf};

use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect, Select};
use jid::BareJid;

use prose_core_client::dtos::{
    DeviceId, Message, MessageId, MessageServerId, ParticipantInfo, PublicRoomInfo, RoomEnvelope,
    SidebarItem, UserId,
};
use prose_core_client::services::{Generic, Room};
use prose_core_client::Client;

use crate::compare_room_envelopes;
use crate::type_display::{
    CompactMessageEnvelope, DeviceInfoEnvelope, JidWithName, ParticipantEnvelope,
};

#[allow(dead_code)]
pub async fn select_contact(client: &Client) -> Result<Option<UserId>> {
    let contacts = client.contact_list.load_contacts().await?.into_iter();
    let contact = select_item_from_list(contacts, |c| JidWithName::from(c.clone())).map(|c| c.id);
    Ok(contact)
}

pub async fn select_contact_or_self(client: &Client) -> Result<Option<UserId>> {
    let contacts = client
        .contact_list
        .load_contacts()
        .await?
        .into_iter()
        .map(JidWithName::from);

    let current_user_id = client.connected_user_id().unwrap().into_user_id();

    let current_user = JidWithName {
        jid: current_user_id.clone().into_inner(),
        name: format!("{} (You)", current_user_id.formatted_username()),
    };

    let user_id = select_item_from_list(
        iter::once(current_user).chain(contacts),
        ToString::to_string,
    )
    .map(|c| c.jid.into());

    Ok(user_id)
}

pub async fn select_device(client: &Client, user_id: &UserId) -> Result<Option<DeviceId>> {
    let devices = client
        .user_data
        .load_user_device_infos(&user_id)
        .await?
        .into_iter()
        .map(|d| DeviceInfoEnvelope(d));
    let device_id = select_item_from_list(devices, ToString::to_string).map(|d| d.0.id);
    Ok(device_id)
}

pub async fn select_multiple_contacts(client: &Client) -> Result<Vec<UserId>> {
    let contacts = client
        .contact_list
        .load_contacts()
        .await?
        .into_iter()
        .map(JidWithName::from);
    Ok(select_multiple_jids_from_list(contacts)
        .into_iter()
        .map(UserId::from)
        .collect())
}

pub async fn select_room(
    client: &Client,
    filter: impl Fn(&SidebarItem) -> bool,
) -> Result<Option<RoomEnvelope>> {
    let mut rooms = client
        .sidebar
        .sidebar_items()
        .await
        .into_iter()
        .filter_map(|room| {
            if !filter(&room) {
                return None;
            }
            Some(room.room)
        })
        .collect::<Vec<_>>();
    rooms.sort_by(compare_room_envelopes);

    if rooms.is_empty() {
        println!("Could not find any matching rooms.");
        return Ok(None);
    }

    Ok(select_item_from_list(rooms, |room| {
        JidWithName::from(room.clone())
    }))
}

pub async fn select_muc_room(client: &Client) -> Result<Option<RoomEnvelope>> {
    select_room(client, |room| {
        if let RoomEnvelope::DirectMessage(_) = room.room {
            return false;
        }
        true
    })
    .await
}

pub async fn select_participant<T>(room: &Room<T>) -> Option<ParticipantInfo> {
    select_item_from_list(room.participants(), |p| ParticipantEnvelope(p.clone()))
}

pub async fn select_message(room: &Room<Generic>) -> Result<Option<MessageId>> {
    let messages = load_messages(room, 1).await?;
    let message =
        select_item_from_list(messages, |message| CompactMessageEnvelope(message.clone()));
    Ok(message.map(|m| m.id))
}

pub async fn select_public_channel(client: &Client) -> Result<Option<PublicRoomInfo>> {
    let rooms = client.rooms.load_public_rooms().await?;
    Ok(select_item_from_list(rooms, |room| JidWithName::from(room.clone())).clone())
}

pub async fn select_sidebar_item(client: &Client) -> Result<Option<SidebarItem>> {
    let items = client.sidebar.sidebar_items().await;
    if items.is_empty() {
        return Ok(None);
    }
    Ok(select_item_from_list(items, |item| JidWithName::from(item.clone())).clone())
}

pub fn select_item_from_list<T, O: ToString>(
    iter: impl IntoIterator<Item = T>,
    format: impl Fn(&T) -> O,
) -> Option<T> {
    let mut list = iter.into_iter().collect::<Vec<_>>();
    let display_list = list.iter().map(format).collect::<Vec<_>>();
    let Some(selection) = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(display_list.as_slice())
        .interact_opt()
        .unwrap()
    else {
        return None;
    };
    println!();
    Some(list.swap_remove(selection))
}

pub fn select_multiple_jids_from_list(jids: impl IntoIterator<Item = JidWithName>) -> Vec<BareJid> {
    let items = jids.into_iter().collect::<Vec<JidWithName>>();
    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select contacts")
        .items(items.as_slice())
        .interact()
        .unwrap();
    println!();
    selection
        .into_iter()
        .map(|idx| items[idx].jid.clone())
        .collect()
}

pub fn select_file(prompt: &str) -> Option<PathBuf> {
    let path = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .validate_with({
            |input: &String| {
                if input.is_empty() {
                    return Ok(());
                }
                if Path::new(input.trim()).exists() {
                    Ok(())
                } else {
                    Err("No file exists at the given path")
                }
            }
        })
        .allow_empty(true)
        .interact_text()
        .unwrap();

    if path.is_empty() {
        return None;
    }

    Some(Path::new(path.trim()).to_path_buf())
}

pub async fn load_messages(room: &Room<Generic>, pages: u32) -> Result<Vec<Message>> {
    let mut stanza_id: Option<MessageId> = None;
    let mut messages = vec![];
    let mut page = 0;

    loop {
        let result_set = if let Some(stanza_id) = &stanza_id {
            room.load_messages_before(stanza_id).await
        } else {
            room.load_latest_messages().await
        }?;

        stanza_id = result_set.last_message_id.clone();
        messages.extend(&mut result_set.into_iter().rev());
        page += 1;

        if page == pages || stanza_id.is_none() {
            break;
        }
    }

    Ok(messages)
}
