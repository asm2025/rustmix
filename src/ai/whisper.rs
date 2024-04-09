use anyhow::{Ok, Result};
use futures::stream::StreamExt;
use rodio::Decoder;
use rwhisper::WhisperBuilder;
pub use rwhisper::{Segment, WhisperLanguage, WhisperSource};
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::UnboundedSender;

pub struct Whisper {
    model: rwhisper::Whisper,
}

impl Whisper {
    pub fn new() -> Self {
        let model = WhisperBuilder::default()
            .with_source(WhisperSource::TinyEn)
            .build()
            .unwrap();
        Whisper { model }
    }

    pub fn with(source: WhisperSource, language: WhisperLanguage, cpu_only: bool) -> Self {
        let model = WhisperBuilder::default()
            .with_source(source)
            .with_language(Some(language))
            .with_cpu(cpu_only)
            .build()
            .unwrap();
        Whisper { model }
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
