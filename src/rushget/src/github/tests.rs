use std::path::Path;
use log::LevelFilter;
use rstest::*;
use crate::components::config::{ConfigLoader, DEFAULT_CONFIG_YAML};
use super::*;
use std::fs::remove_file;

#[fixture]
fn init_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

#[rstest]
#[tokio::test]
async fn download_success(init_logger: ()) {
    let loader = ConfigLoader::default();
    let config = loader.load_config_yaml(DEFAULT_CONFIG_YAML).unwrap();
    let release_url = "https://github.com/Amazing-Favorites/Amazing-Favorites/archive/refs/tags/v0.8.0.zip".to_string();
    let task = GithubReleaseTask::new(config, release_url).run().await;
    assert!(task.is_ok());
    // remove the downloaded file
    let path = Path::new("v0.8.0.zip");
    remove_file(path).unwrap();
}

pub const ERROR_MIRROR_YAML: &str = include_str!("error_mirror.yaml");

#[rstest]
#[tokio::test]
async fn download_failed(init_logger: ()) {
    let loader = ConfigLoader::default();
    info!("{}",ERROR_MIRROR_YAML);
    let config = loader.load_config_yaml(ERROR_MIRROR_YAML).unwrap();
    let release_url = "https://github.com/Amazing-Favorites/Amazing-Favorites/archive/refs/tags/v0.8.0.zip".to_string();
    let task = GithubReleaseTask::new(config, release_url).run().await;
    assert!(task.is_err());
}