use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum DockermirError {
    #[error("the image name is mismatched with all rules")]
    MismatchAllRule,
    #[error("failed to load remote config from url: {0}")]
    FailedToLoadRemoteConfig(String),
    #[error("failed to pull image source: {source_image}, mirror: {mirror_image}, error: {error}")]
    DockerPullError {
        source_image: String,
        mirror_image: String,
        error: String,
    },
    #[error("failed to tag image source: {source_image}, mirror: {mirror_image}, error: {error}")]
    DockerTagError {
        source_image: String,
        mirror_image: String,
        error: String,
    },
    #[error("failed to rmi image: {image}, error: {error}")]
    DockerRemoveImageError {
        image: String,
        error: String,
    },
}
