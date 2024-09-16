use futures::{executor::block_on, stream::StreamExt};
use kalosm::sound::*;
pub use kalosm::sound::{rodio::Decoder, Segment, WhisperLanguage, WhisperSource};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::Result;

#[derive(Clone)]
pub struct Audio {
    model: Arc<Whisper>,
}

impl Audio {
    /// Creates a new `Audio` instance with the Whisper source set to TinyEn and the language set to English.
    /// DO NOT USE THIS FUNCTION IF YOU WANT ACCURATE RESULT.
    pub async fn quick() -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(WhisperSource::TinyEn)
            .with_language(Some(WhisperLanguage::English))
            .build()
            .await?;
        Ok(Audio {
            model: Arc::new(model),
        })
    }

    pub async fn new() -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_language(Some(WhisperLanguage::English))
            .build()
            .await?;
        Ok(Audio {
            model: Arc::new(model),
        })
    }

    pub async fn with_source(source: WhisperSource) -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(source)
            .build()
            .await?;
        Ok(Audio {
            model: Arc::new(model),
        })
    }

    pub async fn with(source: WhisperSource, language: WhisperLanguage) -> Result<Self> {
        let model = WhisperBuilder::default()
            .with_source(source)
            .with_language(Some(language))
            .build()
            .await?;
        Ok(Audio {
            model: Arc::new(model),
        })
    }

    pub fn transcribe_file<T: AsRef<Path>>(&self, file_name: T) -> Result<String> {
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

    pub async fn transcribe_file_async<T: AsRef<Path>>(&self, file_name: T) -> Result<String> {
        let file = File::open(file_name)?;
        let source = Decoder::new(BufReader::new(file))?;
        let mut stream = self.model.transcribe(source)?;
        let mut text = String::new();

        while let Some(result) = stream.next().await {
            text.push_str(result.text());
        }

        Ok(text)
    }

    pub fn transcribe_file_callback<T: AsRef<Path>>(
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

    pub async fn transcribe_file_callback_async<T: AsRef<Path>>(
        &self,
        file_name: T,
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

    pub fn transcribe_stream<T: AsRef<Path>>(
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
