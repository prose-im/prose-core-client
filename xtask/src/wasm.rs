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
    let args = [
        "--target",
        "web",
        "--weak-refs",
        "--scope",
        "prose-org",
        "--release",
    ];

    cmd!(sh, "wasm-pack build")
        .args(args)
        .env(
            "RUSTFLAGS",
            "-C panic=abort -C codegen-units=1 -C opt-level=z",
        )
        .run()?;

    Ok(())
}
