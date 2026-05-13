use binix_core::types::{Url, ResourceType};
use binix_core::Result;
use reqwest::Client;

pub struct NetworkClient {
    client: Client,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder().build().unwrap(),
        }
    }

    pub async fn fetch(&self, url: &str, ty: ResourceType) -> Result<Vec<u8>> {
        let resp = self.client.get(url).send().await?;
        if !url.starts_with("https://") && ty != ResourceType::Image {
            return Err(binix_core::error::BinixError::Security("Mixed content blocked".into()));
        }
        Ok(resp.bytes().await?.to_vec())
    }
}