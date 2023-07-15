use anyhow::Result;
use clap::{Parser, Subcommand};

mod wasm;

#[derive(Parser)]
struct Xtask {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    WasmPack(wasm::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    match Xtask::parse().cmd {
        Command::WasmPack(args) => args.run().await,
    }
}
