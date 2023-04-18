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
        .join("prose_core_lib")
        .join("examples")
        .join(".env");

    dotenvy::from_path(&path).expect(&format!("Missing .env file at {:?}.", path));
}

pub fn load_credentials(idx: u8) -> (FullJid, String) {
    let account_key = format!("ACCOUNT_{}", idx + 1);
    let password_key = format!("PASSWORD_{}", idx + 1);

    let jid = FullJid::from_str(
        &env::var(&account_key).expect(&format!("Missing '{}' in .env", account_key)),
    )
    .expect(&format!("Invalid {} in .env", account_key));
    let password = env::var(&password_key).expect(&format!("Missing '{}' in .env", password_key));

    (jid, password)
}
