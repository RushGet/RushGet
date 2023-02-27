


use crate::error::DockermirError;
use anyhow::Result;
use cmd_lib::run_cmd;

pub(crate) struct DockerExec {}

impl DockerExec {
    pub(crate) fn new() -> DockerExec {
        DockerExec {}
    }
    pub(crate) fn pull(&self, input: &DockermirPullInput) -> Result<(), DockermirError> {
        let mirror_image = input.mirror_image.clone();
        let result = run_cmd!(docker pull $mirror_image);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DockermirError::DockerPullError {
                source_image: input.source_image.clone(),
                mirror_image: input.mirror_image.clone(),
                error: e.to_string(),
            }),
        }
    }
    pub(crate) fn tag(&self, input: &DockermirPullInput) -> Result<(), DockermirError> {
        let source_image = input.source_image.clone();
        let mirror_image = input.mirror_image.clone();
        let result = run_cmd!(docker tag $mirror_image $source_image);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DockermirError::DockerTagError {
                source_image: input.source_image.clone(),
                mirror_image: input.mirror_image.clone(),
                error: e.to_string(),
            }),
        }
    }

    pub(crate) fn rmi(&self, image: &str) -> Result<(), DockermirError> {
        let result = run_cmd!(docker rmi $image);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DockermirError::DockerRemoveImageError {
                image: image.to_owned(),
                error: e.to_string(),
            }),
        }
    }
}

pub(crate) struct DockermirPullInput {
    source_image: String,
    mirror_image: String,
}

impl DockermirPullInput {
    pub(crate) fn new(source_image: String, mirror_image: String) -> DockermirPullInput {
        DockermirPullInput {
            source_image,
            mirror_image,
        }
    }
}