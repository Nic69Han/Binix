use crate::types::{Url, ResourceType};
use crate::Result;

#[async_trait::async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, url: &Url, ty: ResourceType) -> Result<Vec<u8>>;
}

pub trait Renderer: Send + Sync {
    fn render_frame(&mut self, html: &str, css: &[&str]) -> Result<()>;
}