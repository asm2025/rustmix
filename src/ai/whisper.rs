use anyhow::{Ok, Result};
use futures::stream::StreamExt;
use kalosm::audio::*;
pub use kalosm::audio::{Segment, WhisperLanguage, WhisperSource};
use rodio::Decoder;
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::UnboundedSender;

pub struct Whisper {
    model: kalosm::audio::Whisper,
}

impl Whisper {
    pub async fn new() -> Result<Self> {
        let model = kalosm::audio::Whisper::new().await?;
        Ok(Whisper { model })
    }

    pub async fn with(source: WhisperSource, language: WhisperLanguage) -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(source)
            .with_language(Some(language))
            .build()
            .await?;
        Ok(Whisper { model })
    }

    pub async fn transcribe_file(&self, file_name: &str) -> Result<String> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut stream = self.model.transcribe(source)?;
        let mut text = String::new();

        while let Some(result) = stream.next().await {
            text.push_str(result.text());
        }

        Ok(text)
    }

    pub async fn transcribe_file_callback(
        &self,
        file_name: &str,
        callback: impl Fn(&str) -> (),
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut stream = self.model.transcribe(source)?;

        while let Some(result) = stream.next().await {
            callback(result.text());
        }

        Ok(())
    }

    pub async fn transcribe_stream(
        &self,
        file_name: &str,
        sender: UnboundedSender<Segment>,
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        self.model.transcribe_into(source, sender)
    }
}
