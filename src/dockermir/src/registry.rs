#[cfg(test)]
mod tests;

use std::fs;
use serde::Deserialize;
use regex::Regex;
use anyhow::Result;
use thiserror::Error;
use reqwest;
use reqwest::{Client, Method};
use crate::config::{Config, ConfigLoader, LoadConfigOptions, Rule, Ruleset};
use crate::error::DockermirError;

pub(crate) struct MirrorRegistry {}

impl MirrorRegistry {
    pub(crate) fn new() -> MirrorRegistry {
        MirrorRegistry {}
    }

    pub(crate) async fn get_image_mirror(&self, options: GetImageMirrorOptions) -> Result<ImageMirrorData, DockermirError> {
        // Load the configuration
        let loader = ConfigLoader::default();
        let config = loader.load_config(&LoadConfigOptions {
            remote_config_url: options.remote_config_url.to_owned(),
            local_config_path: options.local_config_path.to_owned(),
        }).await?;

        MirrorRegistry::map_mirror_by_configuration(&options.image, &config)
    }

    fn map_mirror_by_configuration(source: &str, config: &Config) -> Result<ImageMirrorData, DockermirError> {
        // Get the ruleset from the configuration
        let ruleset = &config.ruleset;

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
    pub hit_rule: Rule,
    pub hit_ruleset: Ruleset,
    pub source_image: String,
    pub mirror_image: String,
}
