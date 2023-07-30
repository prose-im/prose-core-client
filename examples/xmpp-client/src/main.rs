use anyhow::Result;
use futures::FutureExt;
use tracing::info;

use common::{enable_debug_logging, load_credentials, Level};
use prose_xmpp::mods::{Chat, Profile, Status};
use prose_xmpp::stanza::presence::Show;
use prose_xmpp::{connector, Client, Event};

// This example starts a XMPP client and listens for messages. If a message is received it loads
// the sender's vCard and response with a greeting and some text.

#[tokio::main]
async fn main() -> Result<()> {
    enable_debug_logging(Level::INFO);

    let client = Client::builder()
        .set_connector_provider(Connector::provider())
        .add_mod(Chat::default())
        .add_mod(Profile::default())
        .add_mod(Status::default())
        .set_event_handler(|client, event| handle_event(client, event).map(|f| f.unwrap()))
        .build();

    let (jid, password) = load_credentials();

    info!("Connecting…");
    client.connect(&jid, password).await?;
    info!("Connected.");

    client
        .get_mod::<Status>()
        .send_presence(Some(Show::Chat), None)?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
        }
        _ = std::future::pending::<()>() => {
            unreachable!()
        }
    }

    Ok(())
}

type Connector = connector::xmpp_rs::Connector;

async fn handle_event(client: Client, event: Event) -> Result<()> {
    let Event::Message(message) = event else {
        return Ok(());
    };

    let Some(from) = message.from else {
        return Ok(());
    };

    let Some(body) = message.body else {
        return Ok(());
    };

    let profile = client.get_mod::<Profile>();
    let chat = client.get_mod::<Chat>();

    let name = profile
        .load_vcard(from.clone())
        .await?
        .map(|vcard| vcard.fn_)
        .and_then(|fn_| fn_.first().map(|fn_| fn_.value.clone()))
        .unwrap_or("<unknown name>".to_string());

    chat.send_message(
        from,
        format!("> {}\nHello {}. This is an automated response.", body, name),
        None,
    )?;

    Ok(())
}
