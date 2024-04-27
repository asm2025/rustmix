use anyhow::Result;
use futures::executor::block_on;
pub use image::{ImageBuffer, Rgb};
use kalosm::vision::Wuerstchen;
pub use kalosm::{vision::WuerstchenInferenceSettings, *};
use std::sync::Arc;

#[derive(Debug)]
pub struct Image {
    model: Arc<Wuerstchen>,
}

impl Image {
    pub async fn new() -> Result<Self> {
        let model = Wuerstchen::builder().build().await?;
        Ok(Image {
            model: Arc::new(model),
        })
    }

    pub async fn with(
        flash_attn: bool,
        decoder_weights: impl Into<String>,
        clip_weights: impl Into<String>,
        prior_clip_weights: impl Into<String>,
        prior_weights: impl Into<String>,
        vqgan_weights: impl Into<String>,
        tokenizer: impl Into<String>,
        prior_tokenizer: impl Into<String>,
    ) -> Result<Self> {
        let model = Wuerstchen::builder()
            .with_flash_attn(flash_attn)
            .with_decoder_weights(decoder_weights)
            .with_clip_weights(clip_weights)
            .with_prior_clip_weights(prior_clip_weights)
            .with_prior_weights(prior_weights)
            .with_vqgan_weights(vqgan_weights)
            .with_tokenizer(tokenizer)
            .with_prior_tokenizer(prior_tokenizer)
            .build()
            .await?;
        Ok(Image {
            model: Arc::new(model),
        })
    }

    pub fn generate(&self, prompt: &str) -> Result<Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>> {
        let settings = WuerstchenInferenceSettings::new(prompt);
        let mut stream = self.model.run(settings)?;
        block_on(async move {
            let mut images = Vec::new();

            while let Some(image) = stream.next().await {
                if let Some(buffer) = image.generated_image() {
                    images.push(buffer);
                }
            }

            Ok(images)
        })
    }

    pub async fn generate_async(&self, prompt: &str) -> Result<Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>> {
        let settings = WuerstchenInferenceSettings::new(prompt);
        let mut stream = self.model.run(settings)?;
        let mut images = Vec::new();

        while let Some(image) = stream.next().await {
            if let Some(buffer) = image.generated_image() {
                images.push(buffer);
            }
        }

        Ok(images)
    }
}
