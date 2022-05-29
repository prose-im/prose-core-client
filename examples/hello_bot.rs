// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate prose_core_client;

use std::env;

use log::{self, LevelFilter, Metadata, Record, SetLoggerError};
use prose_core_client::client::{ProseClientBuilder, ProseClientOrigin, ProseClientUnbindReason};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
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
    // Acquire environment variables
    let (test_jid, test_password) = (
        env::var("TEST_JID").expect("TEST_JID"),
        env::var("TEST_PASSWORD").expect("TEST_PASSWORD"),
    );

    // Initialize logger
    let _logger = Logger::init(LevelFilter::Trace);

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
    // TODO: do not block on this, start a thread or something
    client
        .add(&test_jid, &test_password)
        .expect("account added");

    log::debug!("hello bot has added account");

    // Hold on so that account is added and connected
    let account = client.get(&test_jid).expect("account acquired");
    let _broker = account.broker().expect("broker available");

    // Listen for events on account
    log::debug!("hello bot will listen for events...");

    // TODO: register handler to iter on incoming events
    // TODO: spawn in separate thread please, but block execution there
    // TODO

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
