use kalosm::*;

use crate::{ai::*, language::*, Result};

#[derive(Clone)]
pub struct LlamaSource;

impl ModelSource for LlamaSource {
    type Model = language::Llama;
    type Builder = language::LlamaBuilder;

    fn default_size() -> SourceSize {
        SourceSize::Base
    }

    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    async fn new() -> Result<Self::Model> {
        Self::create(Self::default_size()).await
    }

    async fn create(size: SourceSize) -> Result<Self::Model> {
        let source = match size {
            SourceSize::Tiny => language::LlamaSource::llama_3_2_1b_chat(),
            SourceSize::Small => language::LlamaSource::llama_7b_chat(),
            SourceSize::Base => language::LlamaSource::llama_8b_chat(),
            SourceSize::Medium => language::LlamaSource::llama_13b_chat(),
            SourceSize::Large => language::LlamaSource::llama_70b_chat(),
        };
        let model = language::Llama::builder()
            .with_source(source)
            .build()
            .await?;
        Ok(model)
    }
}
