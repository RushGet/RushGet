#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::Write;
use reqwest::Client;
use crate::components::config::RushGetConfig;
use crate::components::RushGetTask;
use crate::error::DockermirError;
use crate::error::DockermirError::GithubReleaseDownloadError;

pub(crate) struct GithubReleaseTask {
    release_url: String,
    config: RushGetConfig,
}

impl GithubReleaseTask {
    pub(crate) fn new(config: RushGetConfig, release_url: String) -> Self {
        GithubReleaseTask {
            release_url,
            config,
        }
    }
}

impl GithubReleaseTask {
    async fn run_core(&self) -> Result<(), String> {
        // get file name of url
        let file_name = self.release_url.split('/').last().unwrap();
        for mirror in &self.config.github.mirrors {
            let url = mirror.replace_template.replace("${release_url}", &self.release_url);
            // download file
            trace!("Downloading release file from url: {}", &url);
            let client = Client::new();
            let response = client.get(&url).send().await;
            if response.is_err() {
                trace!("Failed to download release file from mirror: {}, error: {}", &mirror.name, response.err().unwrap());
                continue;
            }
            let response = response.unwrap();
            if response.status().is_success() {
                let file = File::create(file_name);
                if file.is_err() {
                    return Err(format!("Failed to create file: {}", file_name));
                }
                let mut file = file.unwrap();
                let bytes = response.bytes().await;
                if bytes.is_err() {
                    return Err(format!("Failed to read bytes from response."));
                }
                let write_result = file.write_all(&bytes.unwrap());
                if write_result.is_err() {
                    return Err(format!("Failed to write bytes to file: {}", file_name));
                }
                return Ok(());
            }
        }
        Err("Failed to download release file.".to_string())
    }
}

#[async_trait::async_trait]
impl RushGetTask for GithubReleaseTask {
    async fn run(self) -> Result<(), DockermirError> {
        let result = self.run_core().await;
        if result.is_err() {
            return Err(GithubReleaseDownloadError {
                url: self.release_url,
                error: result.err().unwrap(),
            });
        }
        Ok(())
    }
}