use anyhow::Result;
use async_trait::async_trait;

use super::image_process_context::ImageProcessContext;

#[async_trait]
pub(super) trait ImageProcessStep: Send + Sync {
    async fn execute(&self, context: &mut ImageProcessContext) -> Result<()>;
}
