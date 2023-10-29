// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};

use anyhow::Result;
use common::{enable_debug_logging, load_credentials, Level};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use jid::{BareJid, FullJid, Jid};
use minidom::convert::IntoAttributeValue;
use prose_core_client::app::dtos::{Address, Contact, Message, Occupant};
use prose_core_client::app::services::RoomEnvelope;
use prose_core_client::infra::avatars::FsAvatarCache;
use prose_core_client::{Client, ClientDelegate, ClientEvent, RoomEventType, SqliteDriver};
use prose_xmpp::connector;
use prose_xmpp::mods::muc;
use prose_xmpp::stanza::ConferenceBookmark;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use url::Url;

async fn configure_client() -> Result<(BareJid, Client)> {
    let cache_path = env::current_dir()?
        .join("examples")
        .join("prose-core-client-cli")
        .join("cache");
    fs::create_dir_all(&cache_path)?;

    println!("Cached data can be found at {:?}", cache_path);

    let client = Client::builder()
        .set_connector_provider(connector::xmpp_rs::Connector::provider())
        .set_driver(SqliteDriver::new(&cache_path.join("db.sqlite3")))
        .set_avatar_cache(FsAvatarCache::new(&cache_path.join("Avatar"))?)
        .set_delegate(Some(Box::new(Delegate {})))
        .build()
        .await?;

    let (jid, password) = load_credentials();

    println!("Connecting to server as {}…", jid);
    client.connect(&jid.to_bare(), password).await?;
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
        .items(&options[..])
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

#[derive(Debug)]
struct JidWithName {
    jid: BareJid,
    name: String,
}

impl Display for JidWithName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<30} | {}", self.name.truncate_to(30), self.jid)
    }
}

impl From<RoomEnvelope> for JidWithName {
    fn from(value: RoomEnvelope) -> Self {
        Self {
            jid: value.to_generic_room().jid().clone(),
            name: format!(
                "{} {}",
                value.kind(),
                value
                    .to_generic_room()
                    .name()
                    .unwrap_or("<untitled>")
                    .to_string()
            ),
        }
    }
}

impl From<muc::Room> for JidWithName {
    fn from(value: muc::Room) -> Self {
        Self {
            jid: value.jid.into_bare(),
            name: value.name.as_deref().unwrap_or("<untitled>").to_string(),
        }
    }
}

impl From<Contact> for JidWithName {
    fn from(value: Contact) -> Self {
        Self {
            jid: value.jid,
            name: value.name,
        }
    }
}

struct ConnectedRoomEnvelope(RoomEnvelope);

impl Display for ConnectedRoomEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:<40} | {:<70} | {}",
            self.0.kind(),
            self.0
                .to_generic_room()
                .name()
                .unwrap_or("<untitled>")
                .to_string()
                .truncate_to(40),
            self.0.to_generic_room().jid().to_string().truncate_to(70),
            self.0
                .to_generic_room()
                .subject()
                .as_deref()
                .unwrap_or("<no subject>")
        )
    }
}

#[derive(Debug)]
struct BookmarkEnvelope(ConferenceBookmark);

struct OccupantEnvelope(Occupant);

impl Display for OccupantEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<20} {:<10}",
            self.0
                .jid
                .as_ref()
                .map(|jid| jid.to_string())
                .unwrap_or("<unknown real jid>".to_string())
                .truncate_to(20),
            self.0
                .affiliation
                .clone()
                .into_attribute_value()
                .unwrap_or("<no affiliation>".to_string()),
        )
    }
}

impl Display for BookmarkEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<30} | {:<70} | autojoin: {:?} | nick: {}",
            self.0
                .conference
                .name
                .as_deref()
                .unwrap_or("<untitled>")
                .to_string()
                .truncate_to(30),
            self.0.jid.to_string().truncate_to(70),
            self.0.conference.autojoin,
            self.0.conference.nick.as_deref().unwrap_or("<no nick>")
        )
    }
}

#[allow(dead_code)]
async fn select_contact(client: &Client) -> Result<BareJid> {
    let contacts = client.contacts.load_contacts().await?.into_iter();
    Ok(
        select_item_from_list(contacts, |c| JidWithName::from(c.clone()))
            .jid
            .clone(),
    )
}

async fn select_multiple_contacts(client: &Client) -> Result<Vec<BareJid>> {
    let contacts = client
        .contacts
        .load_contacts()
        .await?
        .into_iter()
        .map(JidWithName::from);
    Ok(select_multiple_jids_from_list(contacts))
}

async fn select_room(client: &Client) -> Result<RoomEnvelope> {
    let mut rooms = client.rooms.connected_rooms();
    rooms.sort_by(compare_room_envelopes);
    Ok(select_item_from_list(rooms, |room| JidWithName::from(room.clone())).clone())
}

async fn select_muc_room(client: &Client) -> Result<RoomEnvelope> {
    let mut rooms = client
        .rooms
        .connected_rooms()
        .into_iter()
        .filter(|room| {
            if let &RoomEnvelope::DirectMessage(_) = room {
                return false;
            }
            true
        })
        .collect::<Vec<_>>();
    rooms.sort_by(compare_room_envelopes);
    Ok(select_item_from_list(rooms, |room| JidWithName::from(room.clone())).clone())
}

fn select_item_from_list<T, O: ToString>(
    iter: impl IntoIterator<Item = T>,
    format: impl Fn(&T) -> O,
) -> T {
    let mut list = iter.into_iter().collect::<Vec<_>>();
    let display_list = list.iter().map(format).collect::<Vec<_>>();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a contact")
        .default(0)
        .items(display_list.as_slice())
        .interact()
        .unwrap();
    println!();
    list.swap_remove(selection)
}

fn select_multiple_jids_from_list(jids: impl IntoIterator<Item = JidWithName>) -> Vec<BareJid> {
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

async fn load_avatar(client: &Client, jid: &BareJid) -> Result<()> {
    println!("Loading avatar for {}…", jid);
    match client.user_data.load_avatar(jid).await? {
        Some(path) => println!("Saved avatar image to {:?}.", path),
        None => println!("{} has not set an avatar.", jid),
    }
    Ok(())
}

async fn save_avatar(client: &Client) -> Result<()> {
    let path = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to image file")
        .validate_with({
            |input: &String| {
                if Path::new(input.trim()).exists() {
                    Ok(())
                } else {
                    Err("No file exists at the given path")
                }
            }
        })
        .interact_text()
        .unwrap();

    client
        .account
        .set_avatar_from_url(Path::new(path.trim()))
        .await?;

    Ok(())
}

async fn load_user_profile(client: &Client, jid: &BareJid) -> Result<()> {
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

async fn update_user_profile(client: &Client, jid: BareJid) -> Result<()> {
    println!("Loading current profile…");
    let mut profile = client
        .user_data
        .load_user_profile(&jid)
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

    client.account.set_profile(&profile).await
}

async fn load_contacts(client: &Client) -> Result<()> {
    let contacts = client.contacts.load_contacts().await?;

    for contact in contacts {
        println!(
            r#"
    Jid: {}
    Name: {}
    Availability: {:?}
    Group: {:?}
    "#,
            contact.jid, contact.name, contact.availability, contact.group,
        );
    }

    Ok(())
}

async fn load_messages(client: &Client) -> Result<()> {
    let room = select_room(client).await?;

    let messages = match room {
        RoomEnvelope::DirectMessage(room) => room.load_latest_messages().await?,
        RoomEnvelope::Group(room) => room.load_latest_messages().await?,
        RoomEnvelope::PrivateChannel(room) => room.load_latest_messages().await?,
        RoomEnvelope::PublicChannel(room) => room.load_latest_messages().await?,
        RoomEnvelope::Generic(room) => room.load_latest_messages().await?,
    };

    for message in messages {
        println!("{}", MessageEnvelope(message));
    }

    Ok(())
}

struct MessageEnvelope(Message);

impl Display for MessageEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {:<36} | {:<20} | {}",
            self.0.timestamp.format("%Y/%m/%d %H:%M:%S"),
            self.0
                .id
                .as_ref()
                .map(|id| id.clone().into_inner())
                .unwrap_or("<no-id>".to_string()),
            self.0.from.to_string().truncate_to(20),
            self.0.body
        )
    }
}

async fn send_message(client: &Client) -> Result<()> {
    let room = select_room(client).await?.to_generic_room();

    let body = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter message")
        .default(String::from("Hello World!"))
        .allow_empty(false)
        .interact_text()
        .unwrap();

    room.send_message(body).await
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
        .cmp(rhs.to_generic_room().name().unwrap_or_default())
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
            RoomEventType::MessagesAppended { message_ids } => {
                let message_id_refs = message_ids.iter().collect::<Vec<_>>();
                let messages = room
                    .to_generic_room()
                    .load_messages_with_ids(message_id_refs.as_slice())
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
    let mut rooms = client.rooms.connected_rooms();
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
    #[strum(serialize = "Load profile")]
    LoadUserProfile,
    #[strum(serialize = "Update profile")]
    UpdateUserProfile,
    #[strum(serialize = "Delete profile")]
    DeleteUserProfile,
    #[strum(serialize = "Load avatar")]
    LoadUserAvatar,
    #[strum(serialize = "Save avatar")]
    SaveUserAvatar,
    #[strum(serialize = "Load contacts")]
    LoadContacts,
    #[strum(serialize = "Add contact")]
    AddContact,
    #[strum(serialize = "Send message")]
    SendMessage,
    #[strum(serialize = "Load messages")]
    LoadMessages,
    #[strum(serialize = "Delete cached data")]
    DeleteCachedData,
    #[strum(serialize = "Create group")]
    CreateGroup,
    #[strum(serialize = "Create public channel")]
    CreatePublicChannel,
    #[strum(serialize = "Create private channel")]
    CreatePrivateChannel,
    #[strum(serialize = "Load public rooms")]
    LoadPublicRooms,
    #[strum(serialize = "Destroy public room")]
    DestroyPublicRoom,
    #[strum(serialize = "Load bookmarks")]
    LoadBookmarks,
    #[strum(serialize = "Delete bookmark")]
    DeleteBookmark,
    #[strum(serialize = "List connected rooms")]
    ListConnectedRooms,
    #[strum(serialize = "Set room subject")]
    SetRoomSubject,
    #[strum(serialize = "List occupants in room")]
    ListRoomOccupants,
    #[strum(serialize = "List members in room")]
    ListRoomMembers,
    Disconnect,
    Noop,
    Exit,
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    enable_debug_logging(Level::TRACE);

    let (jid, client) = configure_client().await?;

    loop {
        println!();

        match select_command() {
            Selection::LoadUserProfile => {
                let jid = prompt_bare_jid(&jid);
                load_user_profile(&client, &jid).await?;
            }
            Selection::UpdateUserProfile => {
                update_user_profile(&client, jid.clone()).await?;
            }
            Selection::DeleteUserProfile => {
                client.account.delete_profile().await?;
            }
            Selection::LoadUserAvatar => {
                let jid = prompt_bare_jid(&jid);
                load_avatar(&client, &jid).await?;
            }
            Selection::SaveUserAvatar => {
                save_avatar(&client).await?;
            }
            Selection::LoadContacts => {
                load_contacts(&client).await?;
            }
            Selection::AddContact => {
                let jid = prompt_bare_jid(None);
                client.contacts.add_contact(&jid).await?;
            }
            Selection::SendMessage => {
                send_message(&client).await?;
            }
            Selection::LoadMessages => {
                load_messages(&client).await?;
            }
            Selection::DeleteCachedData => {
                println!("Cleaning cache…");
                todo!()
                //client.delete_cached_data().await?;
            }
            Selection::CreateGroup => {
                let contacts = select_multiple_contacts(&client).await?;
                client
                    .rooms
                    .create_room_for_direct_message(contacts.as_slice())
                    .await?;
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
                client.rooms.destroy_room(&rooms[selection].jid).await?;
            }
            Selection::LoadBookmarks => {
                todo!()
                // let bookmarks = client
                //     .load_bookmarks_dbg()
                //     .await?
                //     .into_iter()
                //     .map(BookmarkEnvelope)
                //     .map(|b| b.to_string())
                //     .collect::<Vec<_>>();
                //
                // println!("Bookmarks:\n{}", bookmarks.join("\n"));
            }
            Selection::DeleteBookmark => {
                todo!()
                // let bookmarks = client
                //     .load_bookmarks_dbg()
                //     .await?
                //     .into_iter()
                //     .map(BookmarkEnvelope)
                //     .collect::<Vec<_>>();
                //
                // if bookmarks.is_empty() {
                //     println!("No bookmarks to delete");
                //     continue;
                // }
                //
                // let selection = Select::with_theme(&ColorfulTheme::default())
                //     .with_prompt("Select a bookmark to delete")
                //     .default(0)
                //     .items(bookmarks.as_slice())
                //     .interact()
                //     .unwrap();
                // println!();
                //
                // let selected_bookmark = &bookmarks[selection].0;
                // client.delete_bookmark(&selected_bookmark.jid).await?;
                //
                // if Confirm::new()
                //     .with_prompt(format!(
                //         "Do you want to delete room {} as well?",
                //         selected_bookmark.jid
                //     ))
                //     .interact()?
                // {
                //     println!("Deleting room…");
                //     client
                //         .destroy_room(&selected_bookmark.jid.to_bare())
                //         .await?;
                // }
            }
            Selection::ListConnectedRooms => {
                list_connected_rooms(&client).await?;
            }
            Selection::SetRoomSubject => {
                let room = select_muc_room(&client).await?;
                let subject = prompt_string("Enter a subject:");

                match room {
                    RoomEnvelope::DirectMessage(_) => unreachable!(),
                    RoomEnvelope::Group(room) => room.set_topic(Some(&subject)).await,
                    RoomEnvelope::PrivateChannel(room) => room.set_topic(Some(&subject)).await,
                    RoomEnvelope::PublicChannel(room) => room.set_topic(Some(&subject)).await,
                    RoomEnvelope::Generic(room) => room.set_topic(Some(&subject)).await,
                }?;
            }
            Selection::ListRoomOccupants => {
                let room = select_muc_room(&client).await?.to_generic_room();
                let occupants = room
                    .occupants_dbg()
                    .into_iter()
                    .map(|o| OccupantEnvelope(o).to_string())
                    .collect::<Vec<_>>();
                println!("{}", occupants.join("\n"))
            }
            Selection::ListRoomMembers => {
                let room = select_muc_room(&client).await?.to_generic_room();
                let members = room
                    .members()
                    .iter()
                    .map(|jid| jid.to_string())
                    .collect::<Vec<_>>();
                println!("{}", members.join("\n"))
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
