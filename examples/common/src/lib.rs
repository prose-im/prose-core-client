use std::env;
use std::str::FromStr;
use std::time::Instant;

use dotenvy;
use jid::{BareJid, FullJid};
use tracing::metadata::LevelFilter;
pub use tracing::Level;
use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub fn enable_debug_logging(max_level: Level) {
    tracing_subscriber::registry()
        .with(OsLogger::new("org.prose", "default").with_filter(LevelFilter::from_level(max_level)))
        .init();
}

pub fn load_credentials() -> (FullJid, String) {
    let jid_arg = env::args()
        .nth(1)
        .and_then(|str| BareJid::from_str(&str).ok());

    let password_arg = env::args().nth(2);

    if let (Some(account_jid), Some(account_password)) = (jid_arg, password_arg) {
        return (
            FullJid {
                domain: account_jid.domain,
                node: account_jid.node,
                resource: format!("cli-{}", Instant::now().elapsed().as_nanos()),
            },
            account_password.to_string(),
        );
    }

    let path = env::current_dir()
        .expect("Cannot determine current directory")
        .join("examples")
        .join(".env");

    dotenvy::from_path(&path).expect(&format!("Missing .env file at {:?}.", path));

    let jid = FullJid::from_str(&env::var("ACCOUNT").expect("Missing 'ACCOUNT' in .env"))
        .expect("Invalid ACCOUNT in .env");
    let password = env::var("PASSWORD").expect("Missing PASSWORD in .env");

    (jid, password)
}
