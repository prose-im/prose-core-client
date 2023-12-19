// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ffi::OsStr;
use std::str::FromStr;
use std::time::Instant;
use std::{env, fs};

use dotenvy;
use jid::{BareJid, FullJid, ResourcePart};
use tracing::metadata::LevelFilter;
pub use tracing::Level;
use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

pub fn enable_debug_logging(max_level: Level) {
    // Get the current executable path
    let exe_path = env::current_exe().expect("Failed to get the current executable path");

    // Extract the file stem (name without extension)
    let exe_stem = exe_path
        .file_stem()
        .and_then(OsStr::to_str)
        .expect("Failed to extract file stem from executable name");

    let oslog_layer =
        OsLogger::new("org.prose", "default").with_filter(LevelFilter::from_level(max_level));

    let log_dir = env::current_dir()
        .expect("Cannot determine current directory")
        .join("examples")
        .join(exe_stem)
        .join("logs");
    let log_filename = format!("{}.log", exe_stem);

    let log_file_path = log_dir.join(&log_filename);
    if log_file_path.exists() {
        _ = fs::remove_file(log_file_path);
    }

    let appender = tracing_appender::rolling::never(log_dir, log_filename);

    let json_layer = tracing_subscriber::fmt::Layer::new()
        .json() // Use the JSON formatter
        .with_writer(appender)
        .with_ansi(false)
        .with_filter(LevelFilter::from_level(max_level));

    Registry::default()
        .with(oslog_layer)
        .with(json_layer)
        .init();
}

pub fn load_credentials() -> (FullJid, String) {
    let jid_arg = env::args()
        .nth(1)
        .and_then(|str| BareJid::from_str(&str).ok());

    let password_arg = env::args().nth(2);

    if let (Some(account_jid), Some(account_password)) = (jid_arg, password_arg) {
        return (
            FullJid::from_parts(
                account_jid.node().as_ref(),
                &account_jid.domain(),
                &ResourcePart::new(&format!("cli-{}", Instant::now().elapsed().as_nanos()))
                    .unwrap(),
            ),
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
