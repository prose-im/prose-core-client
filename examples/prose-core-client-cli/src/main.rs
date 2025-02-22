// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};

use anyhow::{anyhow, format_err, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use jid::{BareJid, FullJid, Jid};
use minidom::Element;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_LENGTH};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::fs::File;
use url::Url;

use common::{enable_debug_logging, load_credentials, Level};
use prose_core_client::dtos::{
    Address, Attachment, AttachmentType, Availability, Avatar, ParticipantId, RoomEnvelope, RoomId,
    SendMessageRequest, SendMessageRequestBody, StringIndexRangeExt, UploadSlot, UserId,
};
use prose_core_client::infra::encryption::{EncryptionKeysRepository, SessionRepository};
use prose_core_client::infra::general::OsRngProvider;
use prose_core_client::FsAvatarRepository;
use prose_core_client::{
    open_store, Client, ClientDelegate, ClientEvent, ClientRoomEventType, PlatformDriver,
    SignalServiceHandle,
};
use prose_xmpp::{connector, mods};

use crate::type_display::{
    ConnectedRoomEnvelope, DeviceInfoEnvelope, JidWithName, MessageEnvelope, ParticipantEnvelope,
    UserBasicInfoEnvelope,
};
use crate::type_selection::{
    load_messages, select_contact, select_contact_or_self, select_device, select_file,
    select_item_from_list, select_message, select_muc_room, select_multiple_contacts,
    select_multiple_jids_from_list, select_participant, select_public_channel, select_room,
    select_sidebar_item,
};

mod type_display;
mod type_selection;

async fn configure_client() -> Result<(BareJid, Client)> {
    let cache_path = env::current_dir()?
        .join("examples")
        .join("prose-core-client-cli")
        .join("cache");
    fs::create_dir_all(&cache_path)?;

    println!("Cached data can be found at {:?}", cache_path);

    let store = open_store(PlatformDriver::new(&cache_path.join("db.sqlite3"))).await?;

    let client = Client::builder()
        .set_connector_provider(connector::xmpp_rs::Connector::provider())
        .set_encryption_service(Arc::new(SignalServiceHandle::new(
            Arc::new(EncryptionKeysRepository::new(store.clone())),
            Arc::new(SessionRepository::new(store.clone())),
            Arc::new(OsRngProvider),
        )))
        .set_store(store)
        .set_avatar_repository(FsAvatarRepository::new(&cache_path.join("Avatar"))?)
        .set_delegate(Some(Box::new(Delegate {})))
        .build();

    let (jid, password) = load_credentials();

    println!("Connecting to server as {}…", jid);
    client
        .connect(&UserId::from(jid.to_bare()), password.into())
        .await?;
    println!("Connected.");

    println!("Starting room observation…");
    client.rooms.start_observing_rooms().await?;
    println!("Done.");

    Ok((jid.into_bare(), client))
}

fn select_command() -> Selection {
    let options: Vec<Selection> = Selection::iter().collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do?")
        .default(0)
        .items(
            options
                .iter()
                .enumerate()
                .map(|(idx, o)| format!("{}. {}", idx + 1, o))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .interact()
        .ok();

    let Some(selection) = selection else {
        return Selection::Noop;
    };

    println!();
    options[selection].clone()
}

fn prompt_bare_jid<'a>(default: impl Into<Option<&'a BareJid>>) -> BareJid {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter jid")
        .validate_with({
            |input: &String| match BareJid::from_str(input) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
        .default(
            default
                .into()
                .map(|jid| jid.to_string())
                .unwrap_or("".to_string()),
        )
        .interact_text()
        .unwrap();
    println!();
    BareJid::from_str(&input).unwrap()
}

#[allow(dead_code)]
fn prompt_full_jid<'a>(default: impl Into<Option<&'a FullJid>>) -> FullJid {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter jid")
        .validate_with({
            |input: &String| match FullJid::from_str(input) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
        .default(
            default
                .into()
                .map(|jid| jid.to_string())
                .unwrap_or("".to_string()),
        )
        .interact_text()
        .unwrap();
    println!();
    FullJid::from_str(&input).unwrap()
}

#[allow(dead_code)]
fn prompt_jid<'a>(default: impl Into<Option<&'a Jid>>) -> Jid {
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter (full or bare) jid")
        .validate_with({
            |input: &String| match Jid::from_str(input) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
        .default(
            default
                .into()
                .map(|jid| jid.to_string())
                .unwrap_or("".to_string()),
        )
        .interact_text()
        .unwrap();
    println!();
    Jid::from_str(&input).unwrap()
}

#[derive(Clone)]
struct OptString(Option<String>);

impl FromStr for OptString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 0 && s != "<not set>" {
            Ok(OptString(Some(s.to_owned())))
        } else {
            Ok(OptString(None))
        }
    }
}

impl ToString for OptString {
    fn to_string(&self) -> String {
        match self.0 {
            Some(ref str) => str.to_string(),
            None => "<not set>".to_string(),
        }
    }
}

fn prompt_opt_string(prompt: impl Into<String>, default: Option<String>) -> Option<String> {
    Input::<OptString>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(OptString(default))
        .allow_empty(true)
        .interact_text()
        .unwrap()
        .0
}

fn prompt_string(prompt: impl Into<String>) -> String {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(false)
        .interact_text()
        .unwrap()
}

async fn load_avatar(client: &Client, avatar: &Avatar) -> Result<()> {
    println!("Loading avatar {avatar:?}…");
    match client.user_data.load_avatar(avatar).await? {
        Some(path) => println!("Saved avatar image to {:?}.", path),
        None => println!("No avatar found."),
    }
    Ok(())
}

async fn save_avatar(client: &Client) -> Result<()> {
    let Some(file) = select_file("Path to image file (Press enter to cancel)") else {
        return Ok(());
    };
    client.account.set_avatar_from_url(&file).await?;
    Ok(())
}

async fn upload_file(client: &Client, path: impl AsRef<Path>) -> Result<UploadSlot> {
    let path = path.as_ref();
    let Some(path_str) = path.file_name().and_then(|f| f.to_str()) else {
        return Err(format_err!("Invalid filepath."));
    };
    let metadata = path.metadata()?;

    println!("Requesting upload slot…");
    let slot = client
        .uploads
        .request_upload_slot(path_str, metadata.len(), None)
        .await?;

    let mut headers = HeaderMap::new();
    for header in &slot.upload_headers {
        headers.insert(
            HeaderName::try_from(header.name.clone())?,
            HeaderValue::try_from(header.value.clone())?,
        );
    }
    headers.insert(CONTENT_LENGTH, metadata.len().into());

    let file = File::open(path).await?;

    println!("Uploading file…");
    let response = reqwest::Client::new()
        .put(slot.upload_url.clone())
        .headers(headers)
        .body(file)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Server returned status {}", response.status()));
    }

    Ok(slot)
}

async fn load_user_profile(client: &Client, jid: &UserId) -> Result<()> {
    println!("Loading profile for {}…", jid);
    let profile = client.user_data.load_user_profile(jid).await?;

    let Some(profile) = profile else {
        println!("No profile set for {}", jid);
        return Ok(());
    };

    println!(
        r#"
    First Name: {}
    Last Name: {}
    Nickname: {}
    Org: {}
    Role: {}
    Title: {}
    Email: {}
    Tel: {}
    URL: {}
    Locality: {}
    Country: {}
    "#,
        format_opt(profile.first_name),
        format_opt(profile.last_name),
        format_opt(profile.nickname),
        format_opt(profile.org),
        format_opt(profile.role),
        format_opt(profile.title),
        format_opt(profile.email),
        format_opt(profile.tel),
        format_opt(profile.url),
        format_opt(profile.address.as_ref().and_then(|a| a.locality.as_ref())),
        format_opt(profile.address.as_ref().and_then(|a| a.country.as_ref()))
    );
    Ok(())
}

async fn update_user_profile(client: &Client, id: UserId) -> Result<()> {
    println!("Loading current profile…");
    let mut profile = client
        .user_data
        .load_user_profile(&id)
        .await?
        .unwrap_or_default();

    profile.first_name = prompt_opt_string("First name", profile.first_name);
    profile.last_name = prompt_opt_string("Last name", profile.last_name);
    profile.nickname = prompt_opt_string("Nickname", profile.nickname);
    profile.org = prompt_opt_string("Org", profile.org);
    profile.role = prompt_opt_string("Role", profile.role);
    profile.title = prompt_opt_string("Title", profile.title);
    profile.email = prompt_opt_string("Email", profile.email);
    profile.tel = prompt_opt_string("Tel", profile.tel);
    profile.url = prompt_opt_string("URL", profile.url.map(|url| url.into()))
        .and_then(|url| Url::parse(&url).ok());

    let locality = prompt_opt_string(
        "Locality",
        profile.address.as_ref().and_then(|a| a.locality.clone()),
    );
    let country = prompt_opt_string(
        "Country",
        profile.address.as_ref().and_then(|a| a.country.clone()),
    );

    if locality.is_some() || country.is_some() {
        profile.address = Some(Address { locality, country })
    }

    client.account.set_profile(profile).await
}

async fn load_contacts(client: &Client) -> Result<()> {
    let contacts = client.contact_list.load_contacts().await?;

    for contact in contacts {
        println!(
            r#"
    Jid: {}
    Name: {}
    Availability: {:?}
    Group: {:?}
    PresenceSubscription: {:?}
    "#,
            contact.id,
            contact.name,
            contact.availability,
            contact.group,
            contact.presence_subscription
        );
    }

    Ok(())
}

async fn send_message(client: &Client) -> Result<()> {
    let Some(room) = select_room(client, |_| true).await? else {
        return Ok(());
    };

    let participant_ids = room
        .to_generic_room()
        .participants()
        .into_iter()
        .filter_map(|p| p.user_id)
        .collect::<Vec<_>>();

    let message: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter message (leave empty to send a file without a message)")
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let mut body = SendMessageRequestBody {
        text: message.into(),
    };

    let mut request = SendMessageRequest {
        body: (!body.text.as_ref().is_empty()).then_some(body),
        attachments: vec![],
    };

    while let Some(file) = select_file("Path to attachment (Press enter to skip)") {
        let slot = upload_file(client, &file).await?;

        request.attachments.push(Attachment {
            r#type: AttachmentType::File,
            url: slot.download_url,
            media_type: slot.media_type,
            file_name: slot.file_name,
            file_size: Some(slot.file_size),
        });
    }

    println!("Sending message…");
    room.to_generic_room().send_message(request).await?;
    Ok(())
}

fn format_opt<T: Display>(value: Option<T>) -> String {
    match value {
        Some(val) => val.to_string(),
        None => "<not set>".to_string(),
    }
}

fn compare_room_envelopes(lhs: &RoomEnvelope, rhs: &RoomEnvelope) -> Ordering {
    fn sort_order(envelope: &RoomEnvelope) -> i32 {
        match envelope {
            RoomEnvelope::DirectMessage(_) => 0,
            RoomEnvelope::Group(_) => 1,
            RoomEnvelope::PrivateChannel(_) => 2,
            RoomEnvelope::PublicChannel(_) => 3,
            RoomEnvelope::Generic(_) => 4,
        }
    }

    let sort_val1 = sort_order(lhs);
    let sort_val2 = sort_order(rhs);

    if sort_val1 < sort_val2 {
        return Ordering::Less;
    } else if sort_val1 > sort_val2 {
        return Ordering::Greater;
    }

    lhs.to_generic_room()
        .name()
        .unwrap_or_default()
        .cmp(&rhs.to_generic_room().name().unwrap_or_default())
}

struct Delegate {}

impl ClientDelegate for Delegate {
    fn handle_event(&self, client: Client, event: ClientEvent) {
        tokio::spawn(async move {
            match Self::_handle_event(client, event).await {
                Ok(_) => (),
                Err(err) => println!("Failed to handle event. {}", err),
            }
        });
    }
}

impl Delegate {
    async fn _handle_event(_client: Client, event: ClientEvent) -> Result<()> {
        let ClientEvent::RoomChanged { room, r#type } = event else {
            return Ok(());
        };

        match r#type {
            ClientRoomEventType::MessagesAppended { message_ids } => {
                let messages = room
                    .to_generic_room()
                    .load_messages_with_ids(&message_ids)
                    .await?;
                for message in messages {
                    println!("Received message:\n{}", MessageEnvelope(message));
                }
            }
            _ => (),
        };
        Ok(())
    }
}

trait ConnectedRoomExt {
    fn kind(&self) -> String;
}

trait StringExt {
    fn truncate_to(&self, new_len: usize) -> String;
}

impl StringExt for String {
    fn truncate_to(&self, new_len: usize) -> String {
        let count = self.chars().count();

        if count <= new_len {
            return self.clone();
        }

        self.chars().take(new_len - 1).chain(once('…')).collect()
    }
}

impl ConnectedRoomExt for RoomEnvelope {
    fn kind(&self) -> String {
        match self {
            RoomEnvelope::DirectMessage(_) => "💬",
            RoomEnvelope::Group(_) => "👥",
            RoomEnvelope::PrivateChannel(_) => "🔒",
            RoomEnvelope::PublicChannel(_) => "🔊",
            RoomEnvelope::Generic(_) => "🌐",
        }
        .to_string()
    }
}

async fn list_connected_rooms(client: &Client) -> Result<()> {
    let mut rooms = client
        .sidebar
        .sidebar_items()
        .await
        .into_iter()
        .map(|item| item.room)
        .collect::<Vec<_>>();
    rooms.sort_by(compare_room_envelopes);

    let rooms = rooms
        .into_iter()
        .map(ConnectedRoomEnvelope)
        .map(|r| r.to_string())
        .collect::<Vec<_>>();
    println!("Connected rooms:\n{}", rooms.join("\n"));
    Ok(())
}

#[derive(EnumIter, Display, Clone)]
enum Selection {
    #[strum(serialize = "Change password")]
    ChangePassword,
    #[strum(serialize = "Load profile")]
    LoadUserProfile,
    #[strum(serialize = "Update profile")]
    UpdateUserProfile,
    #[strum(serialize = "Set Availability")]
    SetAvailability,
    #[strum(serialize = "Load avatar")]
    LoadUserAvatar,
    #[strum(serialize = "Save avatar")]
    SaveUserAvatar,
    #[strum(serialize = "Load contacts")]
    LoadContacts,
    #[strum(serialize = "Add contact")]
    AddContact,
    #[strum(serialize = "Remove contact")]
    RemoveContact,
    #[strum(serialize = "Load Block List")]
    LoadBlockList,
    #[strum(serialize = "Block User")]
    BlockUser,
    #[strum(serialize = "Unblock User")]
    UnblockUser,
    #[strum(serialize = "Clear Block List")]
    ClearBlockList,
    #[strum(serialize = "List presence subscription requests")]
    ListPresenceSubRequests,
    #[strum(serialize = "Send message to contact or room")]
    SendMessageToRoom,
    #[strum(serialize = "Send continuous messages to a room")]
    SendContinuousMessagesToRoom,
    #[strum(serialize = "Send message to anyhow")]
    SendMessageToAnyone,
    #[strum(serialize = "Load messages")]
    LoadMessages,
    #[strum(serialize = "Update message")]
    UpdateMessage,
    #[strum(serialize = "Delete cached data")]
    DeleteCachedData,
    #[strum(serialize = "Start conversation")]
    StartConversation,
    #[strum(serialize = "Create public channel")]
    CreatePublicChannel,
    #[strum(serialize = "Create private channel")]
    CreatePrivateChannel,
    #[strum(serialize = "Load public rooms")]
    LoadPublicRooms,
    #[strum(serialize = "Join public room")]
    JoinPublicRoom,
    #[strum(serialize = "Join room by JID")]
    JoinRoomByJid,
    #[strum(serialize = "Leave room")]
    LeaveRoom,
    #[strum(serialize = "Destroy public room")]
    DestroyPublicRoom,
    #[strum(serialize = "Destroy connected room")]
    DestroyConnectedRoom,
    #[strum(serialize = "List connected rooms")]
    ListConnectedRooms,
    #[strum(serialize = "Rename connected room")]
    RenameConnectedRoom,
    #[strum(serialize = "List sidebar items")]
    ListSidebarItems,
    #[strum(serialize = "Toggle Favorite for sidebar item")]
    ToggleSidebarItemFavorite,
    #[strum(serialize = "Remove sidebar item")]
    RemoveSidebarItem,
    #[strum(serialize = "Set room subject")]
    SetRoomTopic,
    #[strum(serialize = "List participants in room")]
    ListRoomParticipants,
    #[strum(serialize = "Load participant metadata")]
    LoadParticipantMetadata,
    #[strum(serialize = "Resend group invites")]
    ResendGroupInvites,
    #[strum(serialize = "Invite user to private channel")]
    InviteUserToPrivateChannel,
    #[strum(serialize = "Convert group to private channel")]
    ConvertGroupToPrivateChannel,

    #[strum(serialize = "[OMEMO] List user devices")]
    ListUserDevices,
    #[strum(serialize = "[OMEMO] Delete device")]
    DeleteDevice,
    #[strum(serialize = "[OMEMO] Disable OMEMO (Delete all devices)")]
    DisableOMEMO,

    #[strum(serialize = "[Debug] Load bookmarks")]
    LoadBookmarks,
    #[strum(serialize = "[Debug] Delete individual bookmarks")]
    DeleteIndividualBookmarks,
    #[strum(serialize = "[Debug] Delete whole bookmarks PubSub node")]
    DeleteBookmarksPubSubNode,
    #[strum(serialize = "[Debug] Disco anything")]
    DiscoAnything,
    #[strum(serialize = "[Debug] Send raw XML")]
    SendRawXML,
    Disconnect,
    Noop,
    Exit,
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    enable_debug_logging(Level::INFO);

    let (jid, client) = configure_client().await?;

    loop {
        println!();

        match select_command() {
            Selection::ChangePassword => {
                let Some(new_password) = prompt_opt_string("Enter password", None) else {
                    continue;
                };
                client.account.change_password(&new_password).await?;
            }
            Selection::LoadUserProfile => {
                let jid = prompt_bare_jid(&jid);
                load_user_profile(&client, &jid.into()).await?;
            }
            Selection::UpdateUserProfile => {
                update_user_profile(&client, jid.clone().into()).await?;
            }
            Selection::SetAvailability => {
                let Some(availability) = select_item_from_list(
                    vec![
                        Availability::Available,
                        Availability::Away,
                        Availability::DoNotDisturb,
                    ],
                    |a| a.to_string(),
                ) else {
                    continue;
                };
                client.account.set_availability(availability).await?;
            }
            Selection::LoadUserAvatar => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };

                let Some(participant) = select_participant(&room.to_generic_room()).await else {
                    continue;
                };

                let Some(avatar) = participant.avatar else {
                    println!("{} doesn't have an avatar set.", participant.name);
                    continue;
                };

                load_avatar(&client, &avatar).await?;
            }
            Selection::SaveUserAvatar => {
                save_avatar(&client).await?;
            }
            Selection::LoadContacts => {
                load_contacts(&client).await?;
            }
            Selection::AddContact => {
                let jid = prompt_bare_jid(None);
                client.contact_list.add_contact(&jid.into()).await?;
            }
            Selection::RemoveContact => {
                let Some(contact) = select_contact(&client).await? else {
                    continue;
                };
                client.contact_list.remove_contact(&contact).await?;
            }
            Selection::LoadBlockList => {
                let blocked_users = client.block_list.load_block_list().await?;
                if blocked_users.is_empty() {
                    println!("Block List is empty");
                    continue;
                }

                for blocked_user in blocked_users {
                    println!("{}", UserBasicInfoEnvelope(blocked_user))
                }
            }
            Selection::BlockUser => {
                let jid = prompt_bare_jid(None);
                client.block_list.block_user(&jid.into()).await?;
            }
            Selection::UnblockUser => {
                let blocked_users = client.block_list.load_block_list().await?;
                let Some(user) =
                    select_item_from_list(blocked_users, |u| UserBasicInfoEnvelope(u.clone()))
                else {
                    continue;
                };
                client.block_list.unblock_user(&user.id).await?;
            }
            Selection::ClearBlockList => {
                client.block_list.clear_block_list().await?;
            }
            Selection::ListPresenceSubRequests => {
                let requests = client.contact_list.load_presence_sub_requests().await?;
                if requests.is_empty() {
                    println!("No pending presence subscriptions.");
                    continue;
                }

                let Some(req) =
                    select_item_from_list(requests, |req| format!("{} ({})", req.name, req.id))
                else {
                    continue;
                };

                enum Response {
                    Approve,
                    Deny,
                    Cancel,
                }

                impl Display for Response {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        f.write_str(match self {
                            Response::Approve => "Approve",
                            Response::Deny => "Deny",
                            Response::Cancel => "Cancel",
                        })
                    }
                }

                let Some(response) = select_item_from_list(
                    [Response::Approve, Response::Deny, Response::Cancel],
                    |r| r.to_string(),
                ) else {
                    continue;
                };
                match response {
                    Response::Approve => {
                        client
                            .contact_list
                            .approve_presence_sub_request(&req.id)
                            .await?
                    }
                    Response::Deny => {
                        client
                            .contact_list
                            .deny_presence_sub_request(&req.id)
                            .await?
                    }
                    Response::Cancel => {}
                }
            }
            Selection::SendMessageToRoom => {
                send_message(&client).await?;
            }
            Selection::SendContinuousMessagesToRoom => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };

                let mut idx = 1;

                loop {
                    room.to_generic_room()
                        .send_message(SendMessageRequest {
                            body: Some(SendMessageRequestBody {
                                text: format!("Message {idx}").into(),
                            }),
                            attachments: vec![],
                        })
                        .await?;
                    idx += 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
            Selection::SendMessageToAnyone => {
                let jid = prompt_bare_jid(None);
                let room_id = client.rooms.start_conversation(&[jid.into()]).await?;
                let room: RoomEnvelope = client
                    .sidebar
                    .sidebar_items()
                    .await
                    .into_iter()
                    .find(|r| r.room.to_generic_room().jid() == &room_id)
                    .unwrap()
                    .room;
                let body = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter message")
                    .default(String::from("Hello World!"))
                    .allow_empty(false)
                    .interact_text()
                    .unwrap();
                room.to_generic_room()
                    .send_message(SendMessageRequest {
                        body: Some(SendMessageRequestBody { text: body.into() }),
                        attachments: vec![],
                    })
                    .await?;
            }
            Selection::LoadMessages => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };

                let messages = load_messages(&room.to_generic_room(), 0).await?;
                for message in messages {
                    println!("{}", MessageEnvelope(message));
                }
            }
            Selection::UpdateMessage => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };

                let room = room.to_generic_room();
                let Some(message_id) = select_message(&room).await? else {
                    continue;
                };

                let body: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter updated message")
                    .allow_empty(false)
                    .interact_text()
                    .unwrap();

                room.update_message(
                    message_id,
                    SendMessageRequest {
                        body: Some(SendMessageRequestBody { text: body.into() }),
                        attachments: vec![],
                    },
                )
                .await?;
            }
            Selection::DeleteCachedData => {
                println!("Cleaning cache…");
                client.cache.clear_cache().await?;
            }
            Selection::StartConversation => {
                let contacts = select_multiple_contacts(&client).await?;
                if contacts.is_empty() {
                    println!("No contact selected.");
                    continue;
                }
                client.rooms.start_conversation(contacts.as_slice()).await?;
            }
            Selection::CreatePublicChannel => {
                let room_name = prompt_string("Enter a name for the channel:");
                client
                    .rooms
                    .create_room_for_public_channel(room_name)
                    .await?;
            }
            Selection::CreatePrivateChannel => {
                let room_name = prompt_string("Enter a name for the channel:");
                client
                    .rooms
                    .create_room_for_private_channel(room_name)
                    .await?;
            }
            Selection::LoadPublicRooms => {
                let rooms = client
                    .rooms
                    .load_public_rooms()
                    .await?
                    .into_iter()
                    .map(JidWithName::from)
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>();
                println!("{}", rooms.join("\n"));
            }
            Selection::JoinPublicRoom => {
                let Some(room) = select_public_channel(&client).await? else {
                    continue;
                };
                client.rooms.join_room(&room.id, None).await?;
            }
            Selection::JoinRoomByJid => {
                let jid = prompt_bare_jid(None);
                client.rooms.join_room(&jid.into(), None).await?;
            }
            Selection::LeaveRoom => {
                let Some(room) = select_sidebar_item(&client).await? else {
                    continue;
                };
                client
                    .sidebar
                    .remove_from_sidebar(room.room.to_generic_room().jid())
                    .await?;
            }
            Selection::DestroyPublicRoom => {
                let rooms = client
                    .rooms
                    .load_public_rooms()
                    .await?
                    .into_iter()
                    .map(JidWithName::from)
                    .collect::<Vec<_>>();

                if rooms.is_empty() {
                    println!("No rooms to destroy");
                    continue;
                }

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a room to destroy")
                    .default(0)
                    .items(rooms.as_slice())
                    .interact()
                    .unwrap();
                println!();
                client
                    .rooms
                    .destroy_room(&rooms[selection].jid.clone().into())
                    .await?;
            }
            Selection::DestroyConnectedRoom => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };
                client
                    .rooms
                    .destroy_room(room.to_generic_room().muc_id())
                    .await?;
            }
            Selection::ListConnectedRooms => {
                list_connected_rooms(&client).await?;
            }
            Selection::RenameConnectedRoom => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };
                let name = prompt_string("Enter a new name:");

                match room {
                    RoomEnvelope::DirectMessage(_) => println!("Cannot rename DirectMessage"),
                    RoomEnvelope::Group(_) => println!("Cannot rename Group"),
                    RoomEnvelope::PrivateChannel(room) => room.set_name(name).await?,
                    RoomEnvelope::PublicChannel(room) => room.set_name(name).await?,
                    RoomEnvelope::Generic(room) => room.set_name(name).await?,
                }
            }
            Selection::ListSidebarItems => {
                let items = client.sidebar.sidebar_items().await.into_iter().fold(
                    HashMap::new(),
                    |mut map, item| {
                        let category = match item.room {
                            _ if item.is_favorite => "Favorites",
                            RoomEnvelope::DirectMessage(_) => "Direct Messages",
                            RoomEnvelope::Group(_) => "Group",
                            RoomEnvelope::PrivateChannel(_) => "Private Channels",
                            RoomEnvelope::PublicChannel(_) => "Public Channels",
                            RoomEnvelope::Generic(_) => "Generic",
                        };
                        map.entry(category).or_insert_with(Vec::new).push(item);
                        map
                    },
                );

                let mut keys = items.keys().collect::<Vec<_>>();
                keys.sort();

                for key in keys {
                    println!("# {}:", key);
                    let values = items.get(key).unwrap();
                    for value in values {
                        println!(
                            "  - {:<36} | {:<50} | has draft: {} | unread count: {}",
                            value
                                .room
                                .to_generic_room()
                                .name()
                                .unwrap_or("<untitled>".to_string())
                                .truncate_to(36),
                            value
                                .room
                                .to_generic_room()
                                .jid()
                                .to_string()
                                .truncate_to(50),
                            value.has_draft,
                            value.unread_count
                        );
                    }
                }
            }
            Selection::ToggleSidebarItemFavorite => {
                let Some(item) = select_sidebar_item(&client).await? else {
                    continue;
                };
                client
                    .sidebar
                    .toggle_favorite(item.room.to_generic_room().jid())
                    .await?;
            }
            Selection::RemoveSidebarItem => {
                let Some(item) = select_sidebar_item(&client).await? else {
                    continue;
                };
                client
                    .sidebar
                    .remove_from_sidebar(item.room.to_generic_room().jid())
                    .await?;
            }
            Selection::SetRoomTopic => {
                let Some(room) = select_muc_room(&client).await? else {
                    continue;
                };
                let subject = prompt_string("Enter a subject:");

                match room {
                    RoomEnvelope::DirectMessage(_) => unreachable!(),
                    RoomEnvelope::Group(room) => room.set_topic(Some(subject)).await,
                    RoomEnvelope::PrivateChannel(room) => room.set_topic(Some(subject)).await,
                    RoomEnvelope::PublicChannel(room) => room.set_topic(Some(subject)).await,
                    RoomEnvelope::Generic(room) => room.set_topic(Some(subject)).await,
                }?;
            }
            Selection::ListRoomParticipants => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };
                let occupants = room
                    .to_generic_room()
                    .participants()
                    .iter()
                    .map(|o| ParticipantEnvelope(o.clone()).to_string())
                    .collect::<Vec<_>>();
                println!("{}", occupants.join("\n"))
            }
            Selection::LoadParticipantMetadata => {
                let Some(room) = select_room(&client, |_| true).await? else {
                    continue;
                };
                let Some(participant) = select_participant(&room.to_generic_room()).await else {
                    continue;
                };

                let jid = match participant.id {
                    ParticipantId::User(id) => Jid::from(id.into_inner()),
                    ParticipantId::Occupant(id) => Jid::from(id.into_inner()),
                };

                let profile = client.debug.xmpp_client().get_mod::<mods::Profile>();
                let time = profile.load_last_activity(jid).await?;

                println!("> {time:?}");
            }
            Selection::ResendGroupInvites => {
                let Some(RoomEnvelope::Group(room)) = select_room(&client, |item| {
                    if let RoomEnvelope::Group(_) = item.room {
                        return true;
                    }
                    false
                })
                .await?
                else {
                    continue;
                };

                room.resend_invites_to_members().await?;
            }
            Selection::InviteUserToPrivateChannel => {
                let Some(RoomEnvelope::PrivateChannel(room)) = select_room(&client, |item| {
                    if let RoomEnvelope::PrivateChannel(_) = item.room {
                        return true;
                    }
                    false
                })
                .await?
                else {
                    continue;
                };

                let Some(contact) = select_contact(&client).await? else {
                    continue;
                };
                room.invite_users(vec![&contact]).await?;
            }
            Selection::ConvertGroupToPrivateChannel => {
                let Some(RoomEnvelope::Group(room)) = select_room(&client, |item| {
                    if let RoomEnvelope::Group(_) = item.room {
                        return true;
                    }
                    false
                })
                .await?
                else {
                    continue;
                };

                let channel_name = prompt_string("Enter a name for the private channel");
                room.convert_to_private_channel(&channel_name).await?;
            }
            Selection::ListUserDevices => {
                let Some(jid) = select_contact_or_self(&client).await? else {
                    continue;
                };
                let devices = client.user_data.load_user_device_infos(&jid).await?;

                if devices.is_empty() {
                    println!("No devices found.");
                } else {
                    println!(
                        "{}",
                        devices
                            .into_iter()
                            .map(|d| DeviceInfoEnvelope(d).to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                }
            }
            Selection::DeleteDevice => {
                let Some(device_id) =
                    select_device(&client, &client.connected_user_id().unwrap().into_user_id())
                        .await?
                else {
                    continue;
                };
                client.account.delete_device(&device_id).await?;
                println!("Device deleted.")
            }
            Selection::DisableOMEMO => {
                client.account.disable_omemo().await?;
                println!("OMEMO disabled.")
            }
            Selection::LoadBookmarks => {
                let bookmarks = client
                    .debug
                    .load_bookmarks()
                    .await?
                    .into_iter()
                    .map(|b| JidWithName::from(b).to_string())
                    .collect::<Vec<_>>();
                println!("{}", bookmarks.join("\n"));
            }
            Selection::DeleteIndividualBookmarks => {
                let bookmarks = client
                    .debug
                    .load_bookmarks()
                    .await?
                    .into_iter()
                    .map(JidWithName::from)
                    .collect::<Vec<_>>();
                let selected_bookmarks = select_multiple_jids_from_list(bookmarks);
                client
                    .debug
                    .delete_bookmarks(
                        selected_bookmarks
                            .into_iter()
                            .map(|jid| RoomId::Muc(jid.into())),
                    )
                    .await?;
            }
            Selection::DeleteBookmarksPubSubNode => {
                println!("Deleting PubSub node…");
                client.debug.delete_bookmarks_pubsub_node().await?;
            }
            Selection::DiscoAnything => {
                let jid = prompt_jid(None);
                let caps = client.debug.xmpp_client().get_mod::<mods::Caps>();

                let info = caps.query_disco_info(jid.clone(), None).await?;
                let items = caps.query_disco_items(jid, None).await?;

                println!("Info:\n{info:?}\n\nItems:\n{items:?}");
            }
            Selection::SendRawXML => {
                let input = Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter XML")
                    .interact_text()?;
                let element = Element::from_str(&input)?;
                client.debug.xmpp_client().send_raw_stanza(element)?;
            }
            Selection::Disconnect => {
                println!("Disconnecting…");
                client.disconnect().await;
            }
            Selection::Noop => {}
            Selection::Exit => {
                println!("Bye bye!");
                client.disconnect().await;
                return Ok(());
            }
        }
    }
}
