use crate::paths;
use anyhow::Result;
use cargo_swift::package::{run, FeatureOptions, LibTypeArg, Platform};
use cargo_swift::{Config, Mode};
use std::env;
use std::path::Path;

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
        env::set_current_dir(Path::new(paths::BINDINGS).join(paths::bindings::SWIFT))?;

        run(
            Some(vec![Platform::Ios]),
            None,
            Some("ProseSDK".to_string()),
            "ProseSDK".to_string(),
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

        Ok(())
    }
}
