use futures::{executor::block_on, stream::StreamExt};
pub use kalosm::language::LlamaSource;
use kalosm::language::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use crate::{error::*, Result};

const LEN_MIN: u32 = 1000;
const LEN_DEF: u32 = 1000;
const LEN_MAX: u32 = 10000;

#[derive(Clone)]
pub struct Chat {
    model: Arc<Mutex<Llama>>,
}

impl Chat {
    pub async fn quick() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_3_1_8b_chat())
            .build()
            .await?;
        Ok(Chat {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub async fn phi() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::phi_3_5_mini_4k_instruct())
            .build()
            .await?;
        Ok(Chat {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub async fn new() -> Result<Self> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_8b())
            .build()
            .await?;
        Ok(Chat {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub async fn with_source(source: LlamaSource) -> Result<Self> {
        let model = Llama::builder().with_source(source).build().await?;
        Ok(Chat {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub fn query<T: AsRef<str>>(&self, prompt: T, max_len: Option<u32>) -> Result<String> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        block_on(async move {
            let mut stream = model
                .stream_text(prompt)
                .with_max_length(max)
                .await
                .unwrap();
            let mut text = String::new();

            while let Some(result) = stream.next().await {
                text.push_str(&result);
            }

            Ok(text)
        })
    }

    pub async fn query_async<T: AsRef<str>>(
        &self,
        prompt: T,
        max_len: Option<u32>,
    ) -> Result<String> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        let mut stream = model.stream_text(prompt).with_max_length(max).await?;
        let mut text = String::new();

        while let Some(result) = stream.next().await {
            text.push_str(&result);
        }

        Ok(text)
    }

    pub fn callback<T: AsRef<str>>(
        &self,
        prompt: T,
        max_len: Option<u32>,
        callback: impl Fn(&str) -> (),
    ) -> Result<()> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        block_on(async move {
            let mut stream = model
                .stream_text(prompt)
                .with_max_length(max)
                .await
                .unwrap();

            while let Some(result) = stream.next().await {
                callback(&result);
            }

            Ok(())
        })
    }

    pub async fn callback_async<T: AsRef<str>>(
        &self,
        prompt: T,
        max_len: Option<u32>,
        callback: impl Fn(&str) -> (),
    ) -> Result<()> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        let mut stream = model.stream_text(prompt).with_max_length(max).await?;

        while let Some(result) = stream.next().await {
            callback(&result);
        }

        Ok(())
    }

    pub fn stream<T: AsRef<str>>(
        &self,
        prompt: T,
        max_len: Option<u32>,
        sender: UnboundedSender<String>,
    ) -> Result<()> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        block_on(async move {
            let mut stream = model
                .stream_text(prompt)
                .with_max_length(max)
                .await
                .unwrap();

            while let Some(result) = stream.next().await {
                sender.send(result).unwrap();
            }

            Ok(())
        })
    }

    pub async fn use_stream_async<T: AsRef<str>>(
        &self,
        prompt: T,
        max_len: Option<u32>,
        sender: UnboundedSender<String>,
    ) -> Result<()> {
        let prompt = prompt.as_ref();

        if prompt.is_empty() {
            return Err(NoInputError.into());
        }

        let model = self.model.lock().unwrap();
        let max = max_len.unwrap_or(LEN_DEF).clamp(LEN_MIN, LEN_MAX);
        let mut stream = model.stream_text(prompt).with_max_length(max).await?;

        while let Some(result) = stream.next().await {
            sender.send(result)?;
        }

        Ok(())
    }
}
