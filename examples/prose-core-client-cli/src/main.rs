// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use jid::{BareJid, FullJid, Jid};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use url::Url;

use common::{enable_debug_logging, load_credentials, Level};
use prose_core_client::data_cache::sqlite::SQLiteCache;
use prose_core_client::types::{Address, Availability, Contact, Message, MessageId};
use prose_core_client::{CachePolicy, ClientBuilder, FsAvatarCache};
use prose_xmpp::connector;

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
        .build();

    let (jid, password) = load_credentials();

    println!("Connecting to server…");
    client.connect(&jid, password, Availability::Away).await?;
    println!("Connected.");

    Ok((jid.into(), client))
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

struct ContactEnvelope(Contact);

impl Display for ContactEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.0.name, self.0.jid)
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
    Groups: {}
    "#,
            contact.jid,
            contact.name,
            contact.availability,
            contact.groups.join(", "),
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
    let mut oldest_message_id: Option<MessageId> = messages.last().map(|msg| msg.id.clone());
    print_messages(&messages);

    while let Some(message_id) = oldest_message_id {
        let page = client.load_messages_before(&jid, &message_id).await?;
        oldest_message_id = page.items.last().map(|msg| msg.id.clone());
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
    #[strum(serialize = "Query server features")]
    QueryServerFeatures,
    #[strum(serialize = "Delete cached data")]
    DeleteCachedData,
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
            Selection::QueryServerFeatures => {
                client.query_server_features().await?;
            }
            Selection::DeleteCachedData => {
                println!("Cleaning cache…");
                client.delete_cached_data().await?;
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
