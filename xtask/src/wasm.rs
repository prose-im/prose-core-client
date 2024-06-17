// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use anyhow::{anyhow, Result};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use url::Url;
use xshell::{cmd, Shell};

use crate::paths;

#[derive(clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Build {
        /// Create a development build. Enable debug info, and disable optimizations
        #[arg(long)]
        dev: bool,
    },
    Publish,
    BumpPatch,
}

const NPM_SCOPE: &str = "prose-im";
const GH_OWNER: &str = "prose-im";
const GH_REPO: &str = "prose-core-client";

impl Args {
    pub async fn run(self) -> Result<()> {
        let sh = Shell::new()?;
        sh.change_dir(Path::new(paths::BINDINGS).join(paths::bindings::WASM));

        match self.cmd {
            Command::Build { dev } => {
                sh.remove_path("pkg")?;
                run_wasm_pack(
                    &sh,
                    WasmPackCommand::Build {
                        release: !dev,
                        dev,
                        target: WasmPackTarget::Web,
                    },
                )
            }
            Command::Publish => publish(&sh).await,
            Command::BumpPatch => bump_patch(&sh).await,
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

#[allow(dead_code)]
enum WasmPackTarget {
    Bundler,
    NodeJS,
    Web,
    NoModules,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageJson {
    name: String,
    version: String,
    files: Vec<String>,
    module: String,
    browser: Option<String>,
    types: String,
    #[serde(rename = "sideEffects")]
    side_effects: Vec<String>,
    dependencies: HashMap<String, String>,
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

async fn run_release_github(
    github_token: &str,
    version: &str,
    filename: &str,
    file_path: &PathBuf,
) -> Result<()> {
    println!("Uploading release to GitHub…");

    // Read archive metas & contents
    let file_size = std::fs::metadata(file_path)?.len();
    let file = tokio::fs::File::open(file_path).await?;
    let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);
    let client = reqwest::Client::builder().build()?;

    // Create GitHub release
    let github_release = Octocrab::builder()
        .personal_token(github_token.to_string())
        .build()?
        .repos(GH_OWNER, GH_REPO)
        .releases()
        .create(version)
        .target_commitish("master")
        .name(&format!("Version {}", version))
        .draft(false)
        .prerelease(true)
        .send()
        .await?;

    let stripped_upload_url = github_release
        .upload_url
        .strip_suffix("{?name,label}")
        .unwrap_or(&github_release.upload_url);

    let mut release_upload_url = Url::from_str(stripped_upload_url)?;
    release_upload_url.set_query(Some(format!("{}={}", "name", filename).as_str()));

    let github_response = client
        .post(release_upload_url.as_str())
        .header("Content-Type", "application/octet-stream")
        .header("Authorization", format!("token {}", github_token))
        .header("Content-Length", file_size.to_string())
        .body(body)
        .send()
        .await?;

    if github_response.status().is_success() {
        println!("Upload to GitHub complete");

        Ok(())
    } else {
        println!("Upload to GitHub failed");

        Err(anyhow!("{}", github_response.text().await?))
    }
}

fn run_release_npm(sh: &Shell, npm_token: &str, file_path: &PathBuf) -> Result<()> {
    println!("Publishing release to NPM…");

    // Prepare '.npmrc' configuration
    let npmrc_path = env::current_dir()?.join(".npmrc");

    let mut npmrc_file = std::fs::File::create(&npmrc_path).expect("npmrc file create failed");

    npmrc_file
        .write_all(format!("//registry.npmjs.org/:_authToken={}", npm_token).as_bytes())
        .expect("write failed");

    // Publish release to NPM
    let npm_command = cmd!(
        sh,
        "npm publish --provenance --userconfig={npmrc_path} {file_path}"
    )
    .run();

    // Cleanup '.npmrc' configuration
    std::fs::remove_file(&npmrc_path).expect("could not remove file");

    match npm_command {
        Ok(_) => {
            println!("Release to NPM complete");

            Ok(())
        }
        Err(err) => {
            println!("Release to NPM failed");

            Err(anyhow!("{}", err))
        }
    }
}

async fn publish(sh: &Shell) -> Result<()> {
    // Read tokens from environment
    let github_token =
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let npm_token = std::env::var("NPM_TOKEN").expect("NPM_TOKEN env variable is required");

    // Build & pack archive contents
    sh.remove_path("pkg")?;

    run_wasm_pack(
        &sh,
        WasmPackCommand::Build {
            release: true,
            dev: false,
            target: WasmPackTarget::Web,
        },
    )?;
    run_wasm_pack(&sh, WasmPackCommand::Pack)?;

    // Read package details
    let version = sh
        .read_file("Cargo.toml")?
        .split_once("version = \"")
        .and_then(|it| it.1.split_once('\"'))
        .map(|it| it.0)
        .ok_or_else(|| anyhow::format_err!("can't find version field in the manifest"))?
        .to_owned();

    // Generate archive file name
    let filename = format!("{}-{}-{}.tgz", NPM_SCOPE, paths::bindings::WASM, &version);

    let file_path = env::current_dir()?
        .join(paths::BINDINGS)
        .join(paths::bindings::WASM)
        .join("pkg")
        .join(&filename);

    // Upload release archive to GitHub
    run_release_github(&github_token, &version, &filename, &file_path).await?;

    // Upload release archive to NPM
    run_release_npm(sh, &npm_token, &file_path)?;

    Ok(())
}

async fn bump_patch(sh: &Shell) -> Result<()> {
    let manifest_path = env::current_dir()?
        .join(paths::BINDINGS)
        .join(paths::bindings::WASM)
        .join("Cargo.toml");

    // Read Cargo.toml
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let mut manifest = manifest_content.parse::<toml_edit::DocumentMut>()?;

    // Get current version
    let version_str = manifest["package"]["version"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing version in {}.", manifest_path.display()))?;
    let mut version = version_str.parse::<semver::Version>()?;

    // Increment patch version
    version.patch += 1;

    // Update the version in the manifest
    manifest["package"]["version"] = toml_edit::value(version.to_string());
    fs::write(&manifest_path, manifest.to_string())?;

    let version = version.to_string();
    let manifest_path = manifest_path.display().to_string();
    let commit_message = format!("chore(sdk-js): Bump version to {version}");

    // Commit changes
    cmd!(sh, "git add {manifest_path}").run()?;
    cmd!(sh, "git commit -m {commit_message}").run()?;

    // Create a Git tag
    cmd!(sh, "git tag {version}").run()?;

    Ok(())
}
