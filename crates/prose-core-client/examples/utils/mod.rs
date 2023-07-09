use std::env;
use std::str::FromStr;

use dotenvy;
use jid::FullJid;
use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[allow(dead_code)]
pub fn enable_debug_logging(max_level: Level) {
    tracing_subscriber::registry()
        .with(OsLogger::new("org.prose", "default").with_filter(LevelFilter::from_level(max_level)))
        .init();
}

pub fn load_dot_env() {
    let path = env::current_dir()
        .expect("Cannot determine current directory")
        .join("prose-core-client")
        .join("examples")
        .join(".env");

    dotenvy::from_path(&path).expect(&format!("Missing .env file at {:?}.", path));
}

pub fn load_credentials() -> (FullJid, String) {
    let jid = FullJid::from_str(&env::var("ACCOUNT").expect("Missing 'ACCOUNT' in .env"))
        .expect("Invalid ACCOUNT in .env");
    let password = env::var("PASSWORD").expect("Missing PASSWORD in .env");

    (jid, password)
}
