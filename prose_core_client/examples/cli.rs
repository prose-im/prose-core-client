use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};

use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use jid::{BareJid, FullJid, Jid};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tracing::{span, Level};
use url::Url;
use uuid::Uuid;

use prose_core_client::types::Address;
use prose_core_client::{CachePolicy, ClientBuilder, FsAvatarCache, SQLiteCache};
use prose_core_domain::{Availability, Contact, Message, MessageId};

use crate::utilities::{enable_debug_logging, load_credentials, load_dot_env};

#[path = "utils/mod.rs"]
mod utilities;

type Client = prose_core_client::Client<SQLiteCache, FsAvatarCache>;

async fn configure_client() -> anyhow::Result<(BareJid, Client)> {
    let jid_arg = env::args()
        .nth(1)
        .and_then(|str| BareJid::from_str(&str).ok());

    // Allow passing in a bare jid argument and prompt for a password or load jid and password
    // from .env file otherwise.
    let (account_jid, account_password) = match jid_arg {
        Some(jid) => (
            FullJid {
                domain: jid.domain,
                node: jid.node,
                resource: format!("cli-{}", Uuid::new_v4().to_string()),
            },
            Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Password")
                .interact()
                .unwrap(),
        ),
        None => {
            load_dot_env();
            load_credentials()
        }
    };

    let cache_path = env::current_dir()?
        .join("prose_core_client")
        .join("examples")
        .join("cache");
    fs::create_dir_all(&cache_path)?;

    println!("Cached data can be found at {:?}", cache_path);

    let data_cache = SQLiteCache::open(&cache_path)?;
    let image_cache = FsAvatarCache::new(&cache_path.join("Avatar"))?;

    let client = ClientBuilder::<SQLiteCache, FsAvatarCache>::new()
        .set_data_cache(data_cache)
        .set_avatar_cache(image_cache)
        .build();

    println!("Connecting to server…");
    client
        .connect(&account_jid, account_password, Availability::Away, None)
        .await?;
    println!("Connected.");

    Ok((account_jid.into(), client))
}

fn select_command() -> Selection {
    let options: Vec<Selection> = Selection::iter().collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do?")
        .default(0)
        .items(&options[..])
        .interact()
        .unwrap();
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

async fn select_contact(client: &Client) -> anyhow::Result<BareJid> {
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

async fn load_avatar(client: &Client, jid: &BareJid) -> anyhow::Result<()> {
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

async fn save_avatar(client: &Client) -> anyhow::Result<()> {
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

    client.save_avatar(Path::new(path.trim())).await?;

    Ok(())
}

async fn load_user_profile(client: &Client, jid: &BareJid) -> anyhow::Result<()> {
    println!("Loading profile for {}…", jid);
    let profile = client
        .load_profile(jid.clone(), CachePolicy::default())
        .await?;

    let Some(profile) = profile else {
        println!("No profile set for {}", jid);
        return Ok(())
    };

    println!(
        r#"
    Full Name: {}
    Nickname: {}
    Org: {}
    Title: {}
    Email: {}
    Tel: {}
    URL: {}
    Locality: {}
    Country: {}
    "#,
        format_opt(profile.full_name),
        format_opt(profile.nickname),
        format_opt(profile.org),
        format_opt(profile.title),
        format_opt(profile.email),
        format_opt(profile.tel),
        format_opt(profile.url),
        format_opt(profile.address.as_ref().and_then(|a| a.locality.as_ref())),
        format_opt(profile.address.as_ref().and_then(|a| a.country.as_ref()))
    );
    Ok(())
}

async fn update_user_profile(client: &Client, jid: BareJid) -> anyhow::Result<()> {
    println!("Loading current profile…");
    let mut profile = client
        .load_profile(jid, CachePolicy::default())
        .await?
        .unwrap_or_default();

    profile.full_name = prompt_opt_string("Full name", profile.full_name);
    profile.nickname = prompt_opt_string("Nickname", profile.nickname);
    profile.org = prompt_opt_string("Org", profile.org);
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

async fn load_contacts(client: &Client) -> anyhow::Result<()> {
    let contacts = client.load_contacts(CachePolicy::default()).await?;

    for contact in contacts {
        println!(
            r#"
    Jid: {}
    Name: {}
    Avatar: {}
    Availability: {}
    Status: {}
    Groups: {}
    "#,
            contact.jid,
            contact.name,
            contact
                .avatar
                .and_then(|path| path.into_os_string().into_string().ok())
                .unwrap_or("<not set>".to_string()),
            contact.availability,
            format_opt(contact.status),
            contact.groups.join(", "),
        );
    }

    Ok(())
}

async fn load_messages(client: &Client) -> anyhow::Result<()> {
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

async fn send_message(client: &Client) -> anyhow::Result<()> {
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
    Exit,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    enable_debug_logging(Level::TRACE);

    let span = span!(Level::INFO, "start_cli");
    let _enter = span.enter();

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
            Selection::Exit => {
                println!("Bye bye!");
                client.disconnect().await;
                return Ok(());
            }
        }
    }
}
