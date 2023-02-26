#[macro_use]
extern crate log;

use clap::{Command, Parser, Subcommand};

mod error;
mod components;
mod docker;
mod github;

use anyhow::Result;
use log::LevelFilter;
use thiserror::Error;
use error::DockermirError;
use crate::components::config::{ConfigLoader, LoadConfigOptions};
use crate::components::docker_exec::{DockerExec, DockermirPullInput};
use crate::components::RushGetTask;
use crate::docker::{DockerCheckTask, DockerPullTask};
use crate::github::GithubReleaseTask;

/// Tool to help you to pull docker images from mirror instead of mcr.microsoft.com or docker.io
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set the level of verbosity
    #[arg(short, long)]
    verbose: Option<LevelFilter>,

    /// The url of the remote config file
    #[arg(short, long)]
    remote_config_url: Option<String>,

    /// The path of the local config file
    #[arg(short, long)]
    local_config_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum Commands {
    /// Docker commands
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    /// Github commands
    Github {
        #[command(subcommand)]
        command: GithubCommands,
    },
    /// Update dockermir
    SelfUpdate {
        /// The url of the remote metadata file
        #[arg(short, long)]
        metadata_url: Option<String>,

    },
}

#[derive(Subcommand)]
#[derive(Debug)]
enum DockerCommands {
    /// Pull image from mirror
    Pull {
        /// The name of the Docker image to be pull
        image: String,
    },
    /// Check whether the image is matched with any rules
    Check {
        /// The name of the Docker image to be pull
        image: String,
    },
}

#[derive(Subcommand)]
#[derive(Debug)]
enum GithubCommands {
    /// Download release from mirror
    Release {
        /// The url of the release
        url: String,
    }
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
    let loader = ConfigLoader::default();
    let config = loader.load_config(LoadConfigOptions {
        remote_config_url: cli.remote_config_url.to_owned(),
        local_config_path: cli.local_config_path.to_owned(),
    }).await?;

    trace!("cli: {:?}", &cli);
    let result = match &cli.command {
        Commands::Docker { command } => {
            match command {
                DockerCommands::Pull { image } => {
                    DockerPullTask::new(config, image.to_owned())
                        .run()
                        .await
                }
                DockerCommands::Check { image } => {
                    DockerCheckTask::new(config, image.to_owned())
                        .run()
                        .await
                }
            }
        }
        Commands::Github { command } => {
            match command {
                GithubCommands::Release { url } => {
                    GithubReleaseTask::new(config, url.to_owned())
                        .run()
                        .await
                }
            }
        }
        Commands::SelfUpdate { .. } => {
            info!("Self update is not implemented yet, please visit https://github.com/newbe36524/Dockermir");
            Ok(())
        }
    };
    if result.is_ok() {
        info!("Success to run command: {:?}", &cli.command);
        Ok(())
    } else {
        Err(result.err().unwrap())
    }
}

