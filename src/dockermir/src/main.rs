#[macro_use]
extern crate log;

use clap::{Command, Parser, Subcommand};

mod error;
mod components;
mod docker;

use anyhow::Result;
use log::LevelFilter;
use thiserror::Error;
use docker::{GetImageMirrorOptions, MirrorRegistry};
use error::DockermirError;
use crate::components::docker_exec::{DockerExec, DockermirPullInput};
use crate::components::RushGetTask;
use crate::docker::{DockerCheckTask, DockerPullTask};

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
    if let Some(command) = cli.command {
        match command {
            Commands::Pull { image, remote_config_url, local_config_path } => {
                DockerPullTask::new(image.to_owned(), remote_config_url.to_owned(), local_config_path.to_owned())
                    .run()
                    .await
            }
            Commands::Check { image, remote_config_url, local_config_path } => {
                DockerCheckTask::new(image.to_owned(), remote_config_url.to_owned(), local_config_path.to_owned())
                    .run()
                    .await
            }
            Commands::SelfUpdate { .. } => {
                info!("Self update is not implemented yet, please visit https://github.com/newbe36524/Dockermir");
                Ok(())
            }
        }
    } else {
        // TODO print help
        Ok(())
    }
}
