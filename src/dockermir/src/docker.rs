#[cfg(test)]
mod tests;

use regex::Regex;
use crate::components::config::{ConfigLoader, DockerMirrorRule, DockerMirrorRuleset, LoadConfigOptions, RushGetConfig};
use crate::components::docker_exec::{DockerExec, DockermirPullInput};
use crate::components::RushGetTask;
use crate::error::DockermirError;

pub(crate) struct DockerPullTask {
    image: String,
    remote_config_url: Option<String>,
    local_config_path: Option<String>,
}

impl DockerPullTask {
    pub(crate) fn new(image: String, remote_config_url: Option<String>, local_config_path: Option<String>) -> Self {
        DockerPullTask {
            image,
            remote_config_url,
            local_config_path,
        }
    }
}

#[async_trait::async_trait]
impl RushGetTask for DockerPullTask {
    async fn run(self) -> Result<(), DockermirError> {
        let registry = MirrorRegistry::default();
        let mirror_image = registry.get_image_mirror(GetImageMirrorOptions {
            image: self.image.to_string(),
            remote_config_url: self.remote_config_url.to_owned(),
            local_config_path: self.local_config_path.to_owned(),
        }).await?;
        info!("Pull image: {} from mirror: {}", self.image, &mirror_image.mirror_image);
        trace!("hit ruleset: {:?}", &mirror_image.hit_ruleset);
        trace!("hit rule: {:?}", &mirror_image.hit_rule);
        let exec = DockerExec::new();
        exec.pull(&DockermirPullInput::new(self.image.to_string(), mirror_image.mirror_image.to_owned()))?;
        exec.tag(&DockermirPullInput::new(self.image.to_string(), mirror_image.mirror_image.to_owned()))?;
        exec.rmi(&mirror_image.mirror_image)?;
        info!("Successfully pull image: {}", self.image);
        Ok(())
    }
}

pub(crate) struct DockerCheckTask {
    pub(crate) image: String,
    pub(crate) remote_config_url: Option<String>,
    pub(crate) local_config_path: Option<String>,
}

impl DockerCheckTask {
    pub(crate) fn new(image: String, remote_config_url: Option<String>, local_config_path: Option<String>) -> Self {
        DockerCheckTask {
            image,
            remote_config_url,
            local_config_path,
        }
    }
}

#[async_trait::async_trait]
impl RushGetTask for DockerCheckTask {
    async fn run(self) -> Result<(), DockermirError> {
        let registry = MirrorRegistry::default();
        let mirror_image = registry.get_image_mirror(GetImageMirrorOptions {
            image: self.image.to_string(),
            remote_config_url: self.remote_config_url.to_owned(),
            local_config_path: self.local_config_path.to_owned(),
        }).await;
        match mirror_image {
            Ok(mirror_image) => {
                info!("Image match, it will be pull from mirror: {}", &mirror_image.mirror_image);
                trace!("Image: {} is matched with ruleset: {:?}, rule: {:?}", self.image, &mirror_image.hit_ruleset, &mirror_image.hit_rule);
                Ok(())
            }
            Err(e) => {
                error!("Image: {} is not matched with any ruleset, error: {}", self.image, e);
                Ok(())
            }
        }
    }
}


#[derive(Default)]
pub(crate) struct MirrorRegistry {}

impl MirrorRegistry {
    pub(crate) async fn get_image_mirror(&self, options: GetImageMirrorOptions) -> anyhow::Result<ImageMirrorData, DockermirError> {
        // Load the configuration
        let loader = ConfigLoader::default();
        let config = loader.load_config(&LoadConfigOptions {
            remote_config_url: options.remote_config_url.to_owned(),
            local_config_path: options.local_config_path.to_owned(),
        }).await?;

        MirrorRegistry::map_mirror_by_configuration(&options.image, &config)
    }

    fn map_mirror_by_configuration(source: &str, config: &RushGetConfig) -> anyhow::Result<ImageMirrorData, DockermirError> {
        // Get the ruleset from the configuration
        let ruleset = &config.docker.ruleset;

        // Create a regex object for each rule in the ruleset
        for ruleset in ruleset {
            for rule in &ruleset.rules {
                let regex = Regex::new(&rule.match_regex).unwrap();
                if regex.is_match(&source) {
                    let replacement = rule.replace_template.clone();
                    // replace ${mirror_host} into the mirror host
                    // replace ${mirror_namespace} into the mirror namespace
                    let replacement = replacement.replace("${mirror_host}", &ruleset.mirror_host);
                    let replacement = replacement.replace("${mirror_namespace}", &ruleset.mirror_namespace);
                    let mirror = regex.replace(&source, replacement).to_string();
                    return Ok(ImageMirrorData {
                        hit_rule: rule.clone(),
                        hit_ruleset: ruleset.clone(),
                        source_image: source.to_string(),
                        mirror_image: mirror,
                    });
                } else {
                    trace!("Rule {} does not match image {}, regex: {}", rule.name, source, rule.match_regex);
                }
            }
        }
        return Err(DockermirError::MismatchAllRule);
    }
}

pub(crate) struct GetImageMirrorOptions {
    pub image: String,
    pub remote_config_url: Option<String>,
    pub local_config_path: Option<String>,
}


pub(crate) struct ImageMirrorData {
    pub hit_rule: DockerMirrorRule,
    pub hit_ruleset: DockerMirrorRuleset,
    pub source_image: String,
    pub mirror_image: String,
}
