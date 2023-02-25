#[macro_use]
extern crate log;

use clap::{Command, Parser, Subcommand};

mod registry;
mod docker_exec;
mod config;
mod error;

use anyhow::Result;
use log::LevelFilter;
use thiserror::Error;
use error::DockermirError;
use crate::registry::{GetImageMirrorOptions, MirrorRegistry};
use crate::docker_exec::{DockerExec, DockermirPullInput};

/// Tool to help you to pull docker images from mirror instead of mcr.microsoft.com or docker.io
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set the level of verbosity
    #[arg(short, long)]
    verbose: Option<LevelFilter>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum Commands {
    /// Pull image from mirror
    Pull {
        /// The name of the Docker image to be pull
        image: String,

        /// The url of the remote config file
        #[arg(short, long)]
        remote_config_url: Option<String>,

        /// The path of the local config file
        #[arg(short, long)]
        local_config_path: Option<String>,
    },
    /// Check whether the image is matched with any rules
    Check {
        /// The name of the Docker image to be pull
        image: String,

        /// The url of the remote config file
        #[arg(short, long)]
        remote_config_url: Option<String>,

        /// The path of the local config file
        #[arg(short, long)]
        local_config_path: Option<String>,
    },
    /// Update dockermir
    SelfUpdate {
        /// The url of the remote metadata file
        #[arg(short, long)]
        metadata_url: Option<String>,

    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Failed to run dockermir, error: {}", e);
    }
}

async fn run() -> Result<(), DockermirError> {
    let cli: Cli = Cli::parse();

    if let Some(level) = cli.verbose {
        env_logger::builder()
            .filter_level(level)
            .init();
    } else {
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .init();
    }

    trace!("cli: {:?}", cli);

    match &cli.command {
        Some(Commands::Pull { image, remote_config_url, local_config_path }) => {
            let registry = MirrorRegistry::new();
            let mirror_image = registry.get_image_mirror(GetImageMirrorOptions {
                image: image.to_string(),
                remote_config_url: remote_config_url.to_owned(),
                local_config_path: local_config_path.to_owned(),
            }).await?;
            info!("Pull image: {} from mirror: {}", image, &mirror_image.mirror_image);
            trace!("hit ruleset: {:?}", &mirror_image.hit_ruleset);
            trace!("hit rule: {:?}", &mirror_image.hit_rule);
            let exec = DockerExec::new();
            exec.pull(&DockermirPullInput::new(image.to_string(), mirror_image.mirror_image.to_owned()))?;
            exec.tag(&DockermirPullInput::new(image.to_string(), mirror_image.mirror_image.to_owned()))?;
            exec.rmi(&mirror_image.mirror_image)?;
            info!("Successfully pull image: {}", image);
            Ok(())
        }
        Some(Commands::Check { image, remote_config_url, local_config_path }) => {
            let registry = MirrorRegistry::new();
            let mirror_image = registry.get_image_mirror(GetImageMirrorOptions {
                image: image.to_string(),
                remote_config_url: remote_config_url.to_owned(),
                local_config_path: local_config_path.to_owned(),
            }).await;
            match mirror_image {
                Ok(mirror_image) => {
                    info!("Image match, it will be pull from mirror: {}", &mirror_image.mirror_image);
                    trace!("Image: {} is matched with ruleset: {:?}, rule: {:?}", image, &mirror_image.hit_ruleset, &mirror_image.hit_rule);
                    Ok(())
                }
                Err(e) => {
                    error!("Image: {} is not matched with any ruleset, error: {}", image, e);
                    Ok(())
                }
            }
        }
        Some(Commands::SelfUpdate { .. }) => {
            info!("Self update is not implemented yet, please visit https://github.com/newbe36524/Dockermir");
        }
        None => {
            // show help
            Ok(())
        }
    }
}
