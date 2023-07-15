use std::env;
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use octocrab::Octocrab;
use url::Url;
use xshell::{cmd, Shell};

#[derive(clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Dev {},
    Build {},
    Publish {},
}

const BINDINGS_PATH: &str = "bindings";
const CRATE_NAME: &str = "prose-core-client-wasm";
const NPM_SCOPE: &str = "prose-im";
const GH_OWNER: &str = "prose-im";
const GH_REPO: &str = "prose-core-client";

impl Args {
    pub async fn run(self) -> Result<()> {
        let sh = Shell::new()?;
        sh.change_dir(Path::new(BINDINGS_PATH).join(CRATE_NAME));

        match self.cmd {
            Command::Dev {} => run_wasm_pack(
                &sh,
                WasmPackCommand::Build {
                    release: false,
                    dev: true,
                    target: WasmPackTarget::Web,
                },
            ),
            Command::Build {} => run_wasm_pack(
                &sh,
                WasmPackCommand::Build {
                    release: true,
                    dev: false,
                    target: WasmPackTarget::Web,
                },
            ),
            Command::Publish {} => publish(&sh).await,
        }
    }
}

enum WasmPackCommand {
    Build {
        release: bool,
        dev: bool,
        target: WasmPackTarget,
    },
    Pack,
}

enum WasmPackTarget {
    Bundler,
    NodeJS,
    Web,
    NoModules,
}

struct WasmPackArgs {
    command: WasmPackCommand,
    target: Option<WasmPackTarget>,
    release: bool,
}

fn run_wasm_pack(sh: &Shell, cmd: WasmPackCommand) -> Result<()> {
    let mut sh_args: Vec<&str> = vec![];
    let wasm_pack_cmd: &str;

    match cmd {
        WasmPackCommand::Pack => {
            wasm_pack_cmd = "pack";
        }
        WasmPackCommand::Build {
            release,
            dev,
            target,
        } => {
            wasm_pack_cmd = "build";
            sh_args.extend_from_slice(&["--weak-refs", "--scope", NPM_SCOPE]);

            if release {
                sh_args.push("--release")
            }
            if dev {
                sh_args.push("--dev")
            }

            sh_args.push("--target");

            let target_str = match target {
                WasmPackTarget::Bundler => "bundler",
                WasmPackTarget::NodeJS => "nodejs",
                WasmPackTarget::Web => "web",
                WasmPackTarget::NoModules => "no-modules",
            };
            sh_args.push(target_str);
        }
    }

    cmd!(sh, "wasm-pack {wasm_pack_cmd}")
        .args(sh_args)
        .env(
            "RUSTFLAGS",
            "-C panic=abort -C codegen-units=1 -C opt-level=z",
        )
        .run()?;

    Ok(())
}

async fn publish(sh: &Shell) -> Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    run_wasm_pack(
        &sh,
        WasmPackCommand::Build {
            release: true,
            dev: false,
            target: WasmPackTarget::Web,
        },
    )?;
    run_wasm_pack(&sh, WasmPackCommand::Pack)?;

    let manifest = sh.read_file("Cargo.toml")?;
    let version = manifest
        .split_once("version = \"")
        .and_then(|it| it.1.split_once('\"'))
        .map(|it| it.0)
        .ok_or_else(|| anyhow::format_err!("can't find version field in the manifest"))?;

    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;

    let release = octocrab
        .repos(GH_OWNER, GH_REPO)
        .releases()
        .create(version)
        .target_commitish("master")
        .name(&format!("Version {}", version))
        .draft(false)
        .prerelease(true)
        .send()
        .await?;

    let filename = format!("{}-{}-{}.tgz", NPM_SCOPE, CRATE_NAME, version);
    let file_path = env::current_dir()?
        .join(BINDINGS_PATH)
        .join(CRATE_NAME)
        .join("pkg")
        .join(&filename);

    let stripped_upload_url = release
        .upload_url
        .strip_suffix("{?name,label}")
        .unwrap_or(&release.upload_url);

    let mut release_upload_url = Url::from_str(stripped_upload_url)?;
    release_upload_url.set_query(Some(format!("{}={}", "name", filename).as_str()));

    let file_size = std::fs::metadata(&file_path)?.len();
    let file = tokio::fs::File::open(&file_path).await?;
    let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);

    println!("Uploading file…");

    let client = reqwest::Client::builder().build()?;

    let response = client
        .post(release_upload_url.as_str())
        .header("Content-Type", "application/octet-stream")
        .header("Authorization", format!("token {}", token))
        .header("Content-Length", file_size.to_string())
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Upload complete");
        Ok(())
    } else {
        Err(anyhow!("{}", response.text().await?))
    }
}
