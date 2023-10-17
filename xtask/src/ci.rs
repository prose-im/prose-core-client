// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::paths;
use anyhow::Result;
use std::path::Path;
use xshell::{cmd, Shell};

#[derive(clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Wasm,
    WasmStore,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        let sh = Shell::new()?;

        let path = match self.cmd {
            Command::Wasm => paths::tests::INTEGRATION,
            Command::WasmStore => paths::tests::STORE,
        };

        sh.change_dir(Path::new(paths::TESTS).join(path));
        run_wasm_integration_tests(&sh)
    }
}

fn run_wasm_integration_tests(sh: &Shell) -> Result<()> {
    let args = ["--headless", "--firefox"];

    cmd!(sh, "wasm-pack test").args(args).run()?;
    Ok(())
}
