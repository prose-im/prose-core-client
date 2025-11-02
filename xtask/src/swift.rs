use crate::paths;
use anyhow::Result;
use cargo_swift::package::{run, FeatureOptions, LibTypeArg, Platform};
use cargo_swift::{Config, Mode};
use std::env;
use xshell::Shell;

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
        let crate_dir = env::current_dir()?
            .join(paths::BINDINGS)
            .join(paths::bindings::SWIFT);
        env::set_current_dir(&crate_dir)?;

        run(
            Some(vec![Platform::Ios]),
            None,
            Some("ProseSDK".to_string()),
            "ProseCore".to_string(),
            false,
            Config {
                silent: false,
                accept_all: false,
            },
            Mode::Debug,
            LibTypeArg::Static,
            FeatureOptions {
                features: None,
                all_features: false,
                no_default_features: false,
            },
            false,
        )?;

        // Copy Swift files
        let sh = Shell::new()?;
        sh.change_dir(&crate_dir);

        let source_dir = "./Swift";
        let dest_dir = "./ProseSDK/Sources/ProseSDK";

        for entry in sh.read_dir(source_dir)? {
            if entry.extension().and_then(|s| s.to_str()) == Some("swift") {
                let filename = entry.file_name().unwrap();
                let dest_path = format!("{}/{}", dest_dir, filename.to_string_lossy());
                sh.copy_file(&entry, dest_path)?;
            }
        }

        Ok(())
    }
}
