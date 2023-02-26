use crate::error::DockermirError;

pub(crate) mod config;
pub(crate) mod docker_exec;

#[async_trait::async_trait]
pub(crate) trait RushGetTask {
    async fn run(self) -> Result<(), DockermirError>;
}