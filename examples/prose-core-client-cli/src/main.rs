// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use jid::{BareJid, FullJid, Jid};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use url::Url;

use common::{enable_debug_logging, load_credentials, Level};
use prose_core_client::data_cache::sqlite::SQLiteCache;
use prose_core_client::types::{Address, Availability, Contact, Message, MessageId};
use prose_core_client::{
    room::RoomEnvelope as Room, CachePolicy, ClientBuilder, ClientDelegate, ClientEvent,
    FsAvatarCache,
};
use prose_xmpp::connector;
use prose_xmpp::mods::muc;
use prose_xmpp::stanza::ConferenceBookmark;

type Client = prose_core_client::Client<SQLiteCache, FsAvatarCache>;

async fn configure_client() -> Result<(BareJid, Client)> {
    let cache_path = env::current_dir()?
        .join("examples")
        .join("prose-core-client-cli")
        .join("cache");
    fs::create_dir_all(&cache_path)?;

    println!("Cached data can be found at {:?}", cache_path);

    let data_cache = SQLiteCache::open(&cache_path)?;
    let image_cache = FsAvatarCache::new(&cache_path.join("Avatar"))?;

    let client = ClientBuilder::new()
        .set_connector_provider(connector::xmpp_rs::Connector::provider())
        .set_data_cache(data_cache)
        .set_avatar_cache(image_cache)
        .set_delegate(Some(Box::new(Delegate {})))
        .build();

    let (jid, password) = load_credentials();

    println!("Connecting to server as {}…", jid);
    client.connect(&jid, password, Availability::Away).await?;
    println!("Connected.");

    println!("Starting room observation…");
    client.start_observing_rooms().await?;
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
struct ContactEnvelope(Contact);

impl Display for ContactEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.0.name, self.0.jid)
    }
}

#[derive(Debug)]
struct RoomEnvelope(muc::Room);

impl Display for RoomEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.0.name.as_deref().unwrap_or("<untitled>"),
            self.0.jid
        )
    }
}

#[derive(Debug)]
struct BookmarkEnvelope(ConferenceBookmark);

impl Display for BookmarkEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) autojoin: {:?}, nick: {}",
            self.0.conference.name.as_deref().unwrap_or("<untitled>"),
            self.0.jid,
            self.0.conference.autojoin,
            self.0.conference.nick.as_deref().unwrap_or("<no nick>")
        )
    }
}

async fn select_contact(client: &Client) -> Result<BareJid> {
    let items = client
        .load_contacts(CachePolicy::default())
        .await?
        .into_iter()
        .map(ContactEnvelope)
        .collect::<Vec<_>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a contact")
        .default(0)
        .items(&items[..])
        .interact()
        .unwrap();
    println!();
    Ok(items[selection].0.jid.clone())
}

async fn select_multiple_contacts(client: &Client) -> Result<Vec<Contact>> {
    let items = client
        .load_contacts(CachePolicy::default())
        .await?
        .into_iter()
        .map(ContactEnvelope)
        .collect::<Vec<_>>();

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select contacts")
        .items(&items[..])
        .interact()
        .unwrap();
    println!();

    Ok(selection
        .into_iter()
        .map(|idx| items[idx].0.clone())
        .collect())
}

async fn load_avatar(client: &Client, jid: &BareJid) -> Result<()> {
    println!("Loading avatar for {}…", jid);
    match client
        .load_avatar(jid.clone(), CachePolicy::default())
        .await?
    {
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

    client.save_avatar_from_url(Path::new(path.trim())).await?;

    Ok(())
}

async fn load_user_profile(client: &Client, jid: &BareJid) -> Result<()> {
    println!("Loading profile for {}…", jid);
    let profile = client
        .load_user_profile(jid.clone(), CachePolicy::default())
        .await?;

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
        .load_user_profile(jid, CachePolicy::default())
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

    client.save_profile(profile).await
}

async fn load_contacts(client: &Client) -> Result<()> {
    let contacts = client.load_contacts(CachePolicy::default()).await?;

    for contact in contacts {
        println!(
            r#"
    Jid: {}
    Name: {}
    Availability: {}
    Group: {}
    "#,
            contact.jid, contact.name, contact.availability, contact.group,
        );
    }

    Ok(())
}

async fn load_messages(client: &Client) -> Result<()> {
    let jid = select_contact(client).await?;

    fn print_messages(messages: &[Message]) {
        for message in messages {
            println!("{:?}", message);
        }
    }

    let messages = client.load_latest_messages(&jid, None, true).await?;
    let mut oldest_message_id: Option<MessageId> = messages.last().and_then(|msg| msg.id.clone());
    print_messages(&messages);

    while let Some(message_id) = oldest_message_id {
        let page = client.load_messages_before(&jid, &message_id).await?;
        oldest_message_id = page.items.last().and_then(|msg| msg.id.clone());
        print_messages(&page.items);
    }

    Ok(())
}

async fn send_message(client: &Client) -> Result<()> {
    // let jid = select_contact(client).await?;
    let jid = prompt_jid(Some(&Jid::from_str("marc@prose.org").unwrap()));
    let body = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter message")
        .default(String::from("Hello World!"))
        .allow_empty(false)
        .interact_text()
        .unwrap();
    client.send_message(jid, body).await
}

fn format_opt<T: Display>(value: Option<T>) -> String {
    match value {
        Some(val) => val.to_string(),
        None => "<not set>".to_string(),
    }
}

struct Delegate {}

impl ClientDelegate<SQLiteCache, FsAvatarCache> for Delegate {
    fn handle_event(
        &self,
        client: prose_core_client::Client<SQLiteCache, FsAvatarCache>,
        event: ClientEvent,
    ) {
        tokio::spawn(async move {
            match Self::_handle_event(client, event).await {
                Ok(_) => (),
                Err(err) => println!("Failed to handle event. {}", err),
            }
        });
    }
}

impl Delegate {
    async fn _handle_event(
        client: prose_core_client::Client<SQLiteCache, FsAvatarCache>,
        event: ClientEvent,
    ) -> Result<()> {
        match event {
            ClientEvent::MessagesAppended {
                conversation,
                message_ids,
            } => {
                let messages = client
                    .load_messages_with_ids(&conversation, message_ids.as_slice())
                    .await?;
                for message in messages {
                    println!("Message from {}: {}", message.from, message.body);
                }
            }
            _ => (),
        };
        Ok(())
    }
}

struct ConnectedRoomEnvelope(Room<SQLiteCache, FsAvatarCache>);

impl Display for ConnectedRoomEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self.0 {
            Room::DirectMessage(_) => "direct message",
            Room::Group(_) => "group",
            Room::PrivateChannel(_) => "private channel",
            Room::PublicChannel(_) => "public channel",
            Room::Generic(_) => "generic",
        };
        write!(
            f,
            "[{}] {} ({})",
            kind,
            self.0.name().unwrap_or("<untitled>"),
            self.0.jid()
        )
    }
}

async fn list_connected_rooms(client: &Client) -> Result<()> {
    let mut rooms = client.connected_rooms();
    rooms.sort();

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
                client.delete_profile().await?;
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
            Selection::SendMessage => {
                send_message(&client).await?;
            }
            Selection::LoadMessages => {
                load_messages(&client).await?;
            }
            Selection::DeleteCachedData => {
                println!("Cleaning cache…");
                client.delete_cached_data().await?;
            }
            Selection::CreateGroup => {
                let contacts = select_multiple_contacts(&client).await?;
                client
                    .create_group(&contacts.into_iter().map(|c| c.jid).collect::<Vec<_>>()[..])
                    .await?;
            }
            Selection::CreatePublicChannel => {
                let room_name = prompt_string("Enter a name for the channel:");
                client.create_public_channel(room_name).await?;
            }
            Selection::CreatePrivateChannel => {
                let room_name = prompt_string("Enter a name for the channel:");
                client.create_private_channel(room_name).await?;
            }
            Selection::LoadPublicRooms => {
                let rooms = client
                    .load_public_rooms()
                    .await?
                    .into_iter()
                    .map(RoomEnvelope)
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>();
                println!("{}", rooms.join("\n"));
            }
            Selection::DestroyPublicRoom => {
                let rooms = client
                    .load_public_rooms()
                    .await?
                    .into_iter()
                    .map(RoomEnvelope)
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
                    .destroy_room(&rooms[selection].0.jid.to_bare())
                    .await?;
            }
            Selection::LoadBookmarks => {
                let bookmarks = client.load_bookmarks_dbg().await?;

                let bookmarks1 = bookmarks
                    .0
                    .into_iter()
                    .map(BookmarkEnvelope)
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>();

                let bookmarks2 = bookmarks
                    .1
                    .into_iter()
                    .map(BookmarkEnvelope)
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>();

                println!("Old-style bookmarks:\n{}", bookmarks1.join("\n"));
                println!("New-style bookmarks:\n{}", bookmarks2.join("\n"));
            }
            Selection::DeleteBookmark => {
                let bookmarks = client
                    .load_bookmarks_dbg()
                    .await?
                    .0
                    .into_iter()
                    .map(BookmarkEnvelope)
                    .collect::<Vec<_>>();

                if bookmarks.is_empty() {
                    println!("No bookmarks to delete");
                    continue;
                }

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a bookmark to delete")
                    .default(0)
                    .items(bookmarks.as_slice())
                    .interact()
                    .unwrap();
                println!();

                let selected_bookmark = &bookmarks[selection].0;
                client.delete_bookmark(&selected_bookmark.jid).await?;

                if Confirm::new()
                    .with_prompt(format!(
                        "Do you want to delete room {} as well?",
                        selected_bookmark.jid
                    ))
                    .interact()?
                {
                    println!("Deleting room…");
                    client
                        .destroy_room(&selected_bookmark.jid.to_bare())
                        .await?;
                }
            }
            Selection::ListConnectedRooms => {
                list_connected_rooms(&client).await?;
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
