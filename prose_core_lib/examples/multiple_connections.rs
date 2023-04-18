// extern crate prose_core_lib;
//
// use std::sync::Arc;
// use std::thread;
// use std::time::{Duration, SystemTime};
//
// use jid::FullJid;
//
// use prose_core_lib::modules::Chat;
// use prose_core_lib::{Client, ConnectedClient, ConnectionEvent};
//
// use crate::utilities::{enable_debug_logging, load_credentials, load_dot_env};
//
// #[path = "utils/mod.rs"]
// mod utilities;
//
// fn main() {
//     enable_debug_logging();
//     load_dot_env();
//
//     let credentials1 = load_credentials(0);
//     let credentials2 = load_credentials(1);
//
//     let _conn1 = connect(credentials1.0, credentials1.1);
//     let mut conn2: Option<ConfiguredClient> = Some(connect(credentials2.0, credentials2.1));
//
//     let start_time = SystemTime::now();
//     let mut disconnected = false;
//     let disconnect_after = Duration::from_secs(5);
//
//     loop {
//         thread::sleep(Duration::from_millis(1000));
//
//         if !disconnected
//             && SystemTime::now().duration_since(start_time).unwrap() >= disconnect_after
//         {
//             println!("Disconnecting…");
//             disconnected = true;
//             if let Some(conn) = conn2.take() {
//                 conn.client.disconnect();
//             }
//         }
//     }
// }
//
// fn connect(jid: FullJid, password: impl AsRef<str>) -> ConfiguredClient {
//     let chat = Chat::new()
//         .add_message_observer(|msg| {
//             println!("{}", msg);
//         })
//         .unwrap();
//
//     let chat = Arc::new(chat);
//     let cloned_jid = jid.clone();
//
//     let client = Client::new()
//         .set_connection_handler(move |_, event| {
//             println!("{}, {}", cloned_jid, event);
//             if matches!(event, ConnectionEvent::Disconnect { .. }) {
//                 println!("Disconnected {}", cloned_jid);
//             }
//         })
//         .register_module(chat.clone())
//         .connect(&jid, password.as_ref())
//         .unwrap();
//
//     ConfiguredClient {
//         client,
//         _chat: chat,
//     }
// }
//
// struct ConfiguredClient {
//     client: ConnectedClient,
//     _chat: Arc<Chat>,
// }

fn main() {
    println!("This example is not working currently. Migrate it to using tokio.")
}
