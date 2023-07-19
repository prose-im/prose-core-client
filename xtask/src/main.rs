use anyhow::Result;
use clap::{Parser, Subcommand};

mod ci;
mod wasm;

pub(crate) mod paths {
    pub const BINDINGS: &str = "bindings";
    pub const TESTS: &str = "tests";

    pub mod bindings {
        pub const WASM: &str = "prose-core-client-wasm";
    }
    pub mod tests {
        pub const INTEGRATION: &str = "prose-core-integration-tests";
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
