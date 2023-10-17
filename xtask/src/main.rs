// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use clap::{Parser, Subcommand};

mod ci;
mod wasm;

pub(crate) mod paths {
    pub const BINDINGS: &str = "bindings";
    pub const TESTS: &str = "tests";

    pub mod bindings {
        pub const WASM: &str = "prose-sdk-js";
    }
    pub mod tests {
        pub const INTEGRATION: &str = "prose-core-integration-tests";
        pub const STORE: &str = "prose-store-integration-tests";
    }
}

#[derive(Parser)]
struct Xtask {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    WasmPack(wasm::Args),
    CI(ci::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    match Xtask::parse().cmd {
        Command::WasmPack(args) => args.run().await,
        Command::CI(args) => args.run().await,
    }
}
