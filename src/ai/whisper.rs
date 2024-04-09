use anyhow::{Ok, Result};
use futures::stream::StreamExt;
use rodio::Decoder;
use rwhisper::{Segment, WhisperBuilder};
pub use rwhisper::{WhisperLanguage, WhisperSource};
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct Whisper {
    model: rwhisper::Whisper,
}

impl Whisper {
    pub fn new() -> Self {
        let model = WhisperBuilder::default()
            .with_source(WhisperSource::Tiny)
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

    pub async fn transcribe_stream(
        &self,
        file_name: &str,
        sender: UnboundedSender<String>,
    ) -> Result<()> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let (tx, mut rx) = unbounded_channel::<Segment>();
        tokio::spawn(async move {
            while let Some(result) = rx.recv().await {
                sender.send(result.text().to_owned()).unwrap();
            }
        });

        self.model.transcribe_into(source, tx)
    }
}
