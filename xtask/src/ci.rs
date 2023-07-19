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
}

impl Args {
    pub async fn run(self) -> Result<()> {
        let sh = Shell::new()?;
        sh.change_dir(Path::new(paths::TESTS).join(paths::tests::INTEGRATION));

        match self.cmd {
            Command::Wasm => run_wasm_integration_tests(&sh),
        }
    }
}

fn run_wasm_integration_tests(sh: &Shell) -> Result<()> {
    let args = ["--headless", "--firefox"];

    cmd!(sh, "wasm-pack test").args(args).run()?;
    Ok(())
}
