use anyhow::Result;
use xshell::{cmd, Shell};

#[derive(clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Build {},
}

impl Args {
    pub fn run(self) -> Result<()> {
        let sh = Shell::new()?;
        sh.change_dir("bindings/prose-core-client-wasm");

        match self.cmd {
            Command::Build {} => build(&sh),
        }
    }
}

fn build(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "
        wasm-pack build 
            --target web 
            --weak-refs
            --scope prose-org
            --release
    "
    )
    .run()?;

    Ok(())
}
