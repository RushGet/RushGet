use serde::Deserialize;
use reqwest::Client;
use std::fs;
use std::path::Path;
use crate::error::DockermirError;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct DockerMirrorRule {
    pub(crate) name: String,
    pub(crate) match_regex: String,
    pub(crate) replace_template: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct DockerMirrorRuleset {
    pub(crate) name: String,
    pub(crate) mirror_host: String,
    pub(crate) mirror_namespace: String,
    pub(crate) rules: Vec<DockerMirrorRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RushGetGithubConfig {
    pub(crate) mirrors: Vec<GithubMirror>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct GithubMirror {
    pub(crate) name: String,
    pub(crate) replace_template: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RushGetDockerConfig {
    pub(crate) ruleset: Vec<DockerMirrorRuleset>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RushGetConfig {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) github: RushGetGithubConfig,
    pub(crate) docker: RushGetDockerConfig,
}

#[derive(Debug, Default)]
pub struct LoadConfigOptions {
    pub(crate) remote_config_url: Option<String>,
    pub(crate) local_config_path: Option<String>,
}


#[derive(Debug, Default)]
pub struct ConfigLoader {}

impl ConfigLoader {
    pub(crate) fn load_config_yaml(&self, yaml_content: &str) -> anyhow::Result<RushGetConfig> {
        // Parse the configuration from JSON into a Config struct
        let config: RushGetConfig = serde_yaml::from_str(&yaml_content)
            .expect("Failed to parse config file.");

        Ok(config)
    }

    fn load_default_config(&self) -> RushGetConfig {
        self.load_config_yaml(DEFAULT_CONFIG_YAML).unwrap()
    }

    async fn load_config_from_remote_url(&self, url: &str) -> anyhow::Result<RushGetConfig, DockermirError> {
        // send request to load config as json
        // Try to fetch the remote JSON config file
        let client = Client::new();
        let request_builder = client.get(url);
        let http_result = request_builder.send().await;
        if http_result.is_ok() {
            trace!("Loaded config from remote url: {}, http return", url);
            let response = http_result.unwrap();
            if response.status().is_success() {
                trace!("Loaded config from remote url: {}, status code success", url);
                let body = response.text().await;
                if body.is_ok() {
                    trace!("Loaded config from remote url: {}, body is ok", url);
                    let config = self.load_config_yaml(&body.unwrap());
                    if config.is_ok() {
                        trace!("Loaded config from remote url: {}, config loaded is ok", url);
                        return Ok(config.unwrap());
                    } else {
                        error!("Failed to load config from remote url: {}", url);
                    }
                } else {
                    error!("Failed to load config from remote url: {}", url);
                }
            } else {
                error!("Failed to load config from remote url: {}", url);
            }
        }
        return Err(DockermirError::FailedToLoadRemoteConfig(url.to_string()));
    }

    pub(crate) fn load_config_file(&self, file_path: &str) -> anyhow::Result<RushGetConfig> {
        // load config from file
        let config_file_content = fs::read_to_string(file_path)
            .expect("Failed to read config file.");
        let config = self.load_config_yaml(&config_file_content);
        config
    }

    pub(crate) async fn load_config(&self, option: LoadConfigOptions) -> anyhow::Result<RushGetConfig, DockermirError> {
        // load config from remote url
        if option.remote_config_url.is_some() {
            let remote_config_url = option.remote_config_url.as_ref().unwrap();
            let config = self.load_config_from_remote_url(remote_config_url).await;
            if let Ok(config) = config {
                info!("Loaded config from remote url: {}", remote_config_url);
                return Ok(config);
            } else {
                error!("Failed to load config from remote url: {}", remote_config_url);
            }
        }

        // load config from local file
        let config_file_path = if let Some(file_path) = &option.local_config_path {
            if Path::exists(Path::new(file_path)) {
                trace!("file path exists: {}", file_path);
                file_path
            } else {
                warn!("file path not exists: {}, will load from default path: {}", file_path, DEFAULT_CONFIG_FILE_PATH);
                DEFAULT_CONFIG_FILE_PATH
            }
        } else {
            trace!("file path not set, will load from default path: {}", DEFAULT_CONFIG_FILE_PATH);
            DEFAULT_CONFIG_FILE_PATH
        };

        if Path::exists(Path::new(config_file_path)) {
            info!("Loaded config from file: {}", config_file_path);
            if let Ok(config) = self.load_config_file(config_file_path) {
                return Ok(config);
            } else {
                warn!("Failed to load config from file: {}, will load from default config", config_file_path);
            }
        } else {
            trace!("File not exists: {}", config_file_path);
        }

        // load default config
        info!("Loaded default config");
        let config = self.load_default_config();
        Ok(config)
    }
}

const DEFAULT_CONFIG_FILE_PATH: &str = "rushget.yaml";
pub const DEFAULT_CONFIG_YAML: &str = include_str!("default_config.yaml");
