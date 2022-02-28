// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate prose_core_client;

use tokio::runtime::Runtime;
use log::{self, Level, LevelFilter, Metadata, Record, SetLoggerError};
use prose_core_client::client::{ProseClientBuilder, ProseClientOrigin, ProseClientUnbindReason};

const TEST_JID: &'static str = "prose@movim.eu";
const TEST_PASSWORD: &'static str = "prose@movim.eu";

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("({}) - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

impl Logger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_max_level(level);
        log::set_boxed_logger(Box::new(Logger))
    }
}

fn main() {
    // Initialize logger
    let logger = Logger::init(LevelFilter::Trace);

    log::debug!("hello bot starting...");

    // Build client
    let mut client = ProseClientBuilder::new()
        .app(ProseClientOrigin::TestsCLI)
        .build()
        .expect("client built")
        .bind()
        .expect("client bound");

    log::info!("hello bot started");

    // Add account
    client.add(TEST_JID, TEST_PASSWORD).expect("account added");

    log::debug!("hello bot has added account");

    // Hold on so that account is added and connected
    let account = client.get(TEST_JID).expect("account acquired");
    let broker = account.broker().expect("broker available");

    // Listen for events on account
    log::debug!("hello bot will listen for events...");

    // TODO: this is just temporary, this should not involve a runtime
    let runtime = Runtime::new().unwrap();

    runtime.block_on(broker.ingress.listen());

    log::debug!("hello bot will send message...");

    // Send message
    // TODO: send message

    log::debug!("hello bot has sent message");

    // Shutdown client
    client
        .unbind(ProseClientUnbindReason::Bye)
        .expect("client unbound");

    log::info!("hello bot has shut down. bye!");
}
