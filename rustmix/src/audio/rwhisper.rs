use futures::{executor::block_on, stream::StreamExt};
use kalosm::sound::*;
pub use kalosm::sound::{rodio::Decoder, Segment, WhisperLanguage, WhisperSource};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::{ai::SourceSize, Result};

#[derive(Clone)]
pub struct RWhisper {
    model: Arc<Whisper>,
}

impl RWhisper {
    pub async fn quick() -> Result<Self> {
        Self::new(SourceSize::Tiny).await
    }

    pub async fn new(size: SourceSize) -> Result<Self> {
        let source = match size {
            SourceSize::Tiny => WhisperSource::QuantizedTiny,
            SourceSize::Small => WhisperSource::Small,
            SourceSize::Base => WhisperSource::Base,
            SourceSize::Medium => WhisperSource::Medium,
            SourceSize::Large => WhisperSource::QuantizedDistilLargeV3,
        };
        Self::with_source(source).await
    }

    pub async fn with_source(source: WhisperSource) -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(source)
            .build()
            .await?;
        Ok(RWhisper {
            model: Arc::new(model),
        })
    }

    pub async fn with(source: WhisperSource, language: WhisperLanguage) -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(source)
            .with_language(Some(language))
            .build()
            .await?;
        Ok(RWhisper {
            model: Arc::new(model),
        })
    }

    pub fn transcribe<T: AsRef<Path>>(&self, file_name: T) -> Result<String> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut stream = self.model.transcribe(source)?;
        block_on(async move {
            let mut text = String::new();

            while let Some(result) = stream.next().await {
                text.push_str(result.text());
            }

            Ok(text)
        })
    }

    pub async fn transcribe_async<T: AsRef<Path>>(&self, file_name: T) -> Result<String> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut transcription = self.model.transcribe(source);
        let mut text = String::new();

        while let Some(result) = transcription.next().await {
            text.push_str(result.text());
        }

        Ok(text)
    }

    pub fn callback<T: AsRef<Path>>(
        &self,
        file_name: T,
        callback: impl Fn(&str) -> (),
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut stream = self.model.transcribe(source)?;
        block_on(async move {
            while let Some(result) = stream.next().await {
                callback(result.text());
            }
        });
        Ok(())
    }

    pub async fn callback_async<T: AsRef<Path>>(
        &self,
        file_name: T,
        callback: impl Fn(&str) -> (),
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut transcription = self.model.transcribe(source);

        while let Some(result) = transcription.next().await {
            callback(result.text());
        }

        Ok(())
    }

    pub fn stream<T: AsRef<Path>>(
        &self,
        file_name: T,
        sender: UnboundedSender<Segment>,
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        self.model
            .transcribe_into(source, sender)
            .map_err(|e| e.into())
    }
}
