use std::env;

use anyhow::Result;
// use cargo_swift::package::{run, LibTypeArg, Platform};
// use cargo_swift::{Config, Mode};

#[derive(clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Build,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        todo!("FIXME")
        // env::set_current_dir("bindings/prose-sdk-ffi")?;
        //
        // run(
        //     Some(vec![Platform::Macos]),
        //     Some("ProseSDK".to_string()),
        //     false,
        //     Config {
        //         silent: false,
        //         accept_all: false,
        //     },
        //     Mode::Debug,
        //     LibTypeArg::Static,
        //     false,
        // )?;
        //
        // Ok(())
    }
}
