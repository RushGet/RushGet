#[cfg(test)]
mod tests;

use regex::Regex;
use crate::components::config::{ConfigLoader, DockerMirrorRule, DockerMirrorRuleset, LoadConfigOptions, RushGetConfig};
use crate::components::docker_exec::{DockerExec, DockermirPullInput};
use crate::components::RushGetTask;
use crate::error::DockermirError;

pub(crate) struct DockerPullTask {
    image: String,
    config: RushGetConfig,
}

impl DockerPullTask {
    pub(crate) fn new(config: RushGetConfig, image: String) -> Self {
        DockerPullTask {
            image,
            config,
        }
    }
}

#[async_trait::async_trait]
impl RushGetTask for DockerPullTask {
    async fn run(self) -> Result<(), DockermirError> {
        let mirror_image = map_mirror_by_configuration(&self.image, &self.config)?;
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
    image: String,
    config: RushGetConfig,
}

impl DockerCheckTask {
    pub(crate) fn new(config: RushGetConfig, image: String) -> Self {
        DockerCheckTask {
            image,
            config,
        }
    }
}


#[async_trait::async_trait]
impl RushGetTask for DockerCheckTask {
    async fn run(self) -> Result<(), DockermirError> {
        let mirror_image = map_mirror_by_configuration(&self.image, &self.config);
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

pub(crate) struct ImageMirrorData {
    pub hit_rule: DockerMirrorRule,
    pub hit_ruleset: DockerMirrorRuleset,
    pub source_image: String,
    pub mirror_image: String,
}
