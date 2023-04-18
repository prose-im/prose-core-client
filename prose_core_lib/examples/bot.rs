extern crate core;
extern crate prose_core_lib;

use std::sync::Arc;

use jid::BareJid;
use tokio::io::AsyncBufReadExt;
use tracing::{info, Level};

use prose_core_lib::modules::{Profile, Roster, MAM};
use prose_core_lib::{Client, ConnectionEvent};

use crate::utilities::{enable_debug_logging, load_credentials, load_dot_env};

#[path = "utils/mod.rs"]
mod utilities;

#[derive(Clone, Debug)]
struct RosterItem {
    jid: BareJid,
}

async fn select_contact(roster: &Vec<RosterItem>) -> &BareJid {
    let jids: Vec<String> = roster
        .iter()
        .enumerate()
        .map(|(idx, item)| format!("  {}: {}", idx, item.jid.to_string()))
        .collect();

    println!(
        r#"
Select conversation to load messages from:
{}"#,
        jids.join("\n")
    );

    let reader = tokio::io::BufReader::new(tokio::io::stdin());

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        let Ok(idx) = line.parse::<usize>() else {
            println!("Invalid selection {}", line);
            continue
        };

        if idx >= roster.len() {
            println!("Invalid selection {}", line);
            continue;
        }

        return &roster[idx].jid;
    }

    panic!("Something went wrong.")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    enable_debug_logging(Level::TRACE);

    load_dot_env();
    let (account_jid, account_password) = load_credentials(0);

    let roster = Arc::new(Roster::new());
    let mam = Arc::new(MAM::new());
    let profile = Arc::new(Profile::new(None));

    info!("Connecting to server…");
    let client = Client::new()
        .register_module(roster.clone())
        .register_module(mam.clone())
        .register_module(profile.clone())
        .set_connection_handler(|_, event| match event {
            ConnectionEvent::Connect => {
                println!("Connected to server.");
            }
            ConnectionEvent::Disconnect { error } => {
                println!("Connection failed. Reason: {}", error)
            }
        })
        .connect(&account_jid, account_password)
        .await
        .unwrap();

    info!("Loading roster…");

    let items: Vec<RosterItem> = roster
        .load_roster(&client.context())
        .await
        .unwrap()
        .into_iter()
        .filter_map(|item| item.jid().map(|jid| RosterItem { jid }))
        .collect();

    println!("Received roster: {:?}", items);

    let conversation = select_contact(&items).await;

    println!("Loading messages for conversation {}…", conversation);
    let messages: Vec<String> = mam
        .load_messages_in_chat(&client.context(), conversation, None, None, None)
        .await
        .unwrap()
        .0
        .into_iter()
        .map(|m| m.message.to_string())
        .collect();

    println!("{}", messages.join("\n"));

    println!("Loading vCard…");
    profile
        .load_vcard(&client.context(), account_jid.into())
        .await
        .unwrap();

    println!("Exiting…");
    client.disconnect();
}
