use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

pub mod config;
pub mod storage;
pub mod uploader;

use config::Config;
use storage::{StorageBackend, S3Storage};
use uploader::ReleaseUploader;

#[derive(Parser)]
#[command(name = "gold")]
#[command(about = "A release management tool for uploading software packages")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Upload {
        /// Name of the software package
        package_name: String,

        /// Version string (e.g., v1.2.3, v8)
        tag: String,

        /// List of asset files or patterns to upload
        files: Vec<String>,

        /// Storage backend to use
        #[arg(long, default_value = "s3")]
        storage: String,

        /// Additional configuration variables
        #[arg(long = "config", value_parser = parse_config)]
        config_overrides: Vec<(String, String)>,
    },
}

fn parse_config(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Config must be in format KEY=VALUE".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Upload {
            package_name,
            tag,
            files,
            storage,
            config_overrides,
        } => {
            let mut config = Config::from_env()?;

            // Apply config overrides
            for (key, value) in config_overrides {
                config.set_override(&key, &value);
            }

            // Validate configuration before proceeding
            config.validate()
                .with_context(|| "Configuration validation failed")?;

            let storage_backend: Box<dyn StorageBackend> = match storage.as_str() {
                "s3" => Box::new(S3Storage::new(&config).await?),
                "webdav" => {
                    anyhow::bail!("WebDAV storage backend not implemented yet");
                }
                _ => anyhow::bail!("Unknown storage backend: {}", storage),
            };

            let uploader = ReleaseUploader::new(storage_backend);
            uploader.upload_release(&package_name, &tag, &files).await?;

            println!("Successfully uploaded release {} for package {}", tag, package_name);
        }
    }

    Ok(())
}
